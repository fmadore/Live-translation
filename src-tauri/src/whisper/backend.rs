//! Where a session's windows are decoded: the model in this process (on the processor, or on a
//! Mac's graphics processor through Metal) or in the Vulkan helper, and the fallback from the
//! helper to the processor.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;
use whisper_rs::{WhisperContext, WhisperContextParameters, WhisperState};

use super::engine::{self, RawSegment};
use super::models::{self, ModelId, ModelLease, ModelManager};
use super::{cpu, gpu, vad};
use crate::sync::lock;

pub struct LoadedModel {
    /// Where windows are decoded, one at a time across sources. With Both sources, two
    /// concurrent passes would each start a full set of ggml workers, which spin while they
    /// wait and leave capture and the interface fighting for what is left; taking turns
    /// finishes both sooner. The graphics processor helper takes one window at a time anyway.
    inference: Mutex<Backend>,
    pub(super) vad: Option<vad::Vad>,
    lease: ModelLease,
    /// Why the graphics processor the session asked for is not the one decoding, once it is not.
    gpu_failure: Mutex<Option<String>>,
    /// Set when the model moves to the processor mid-session, until the sources have said so.
    pub(super) engine_changed: AtomicBool,
}

/// Where a session's windows are decoded, reported on `whisper-engine` as each source starts
/// and again if the graphics processor fails, so the operator can see it rather than find it in
/// a log the installed app does not write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Engine {
    /// `cpu`, `metal` or `vulkan`.
    pub backend: &'static str,
    /// The graphics processor, as its driver names it: with `vulkan` only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    /// Why the graphics processor the operator left switched on is not in use: technical text
    /// from the driver or the helper, shown as it is under the interface's own sentence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_failure: Option<String>,
}

/// The longest failure shown: the cause comes first, and what follows it is detail.
const MAX_FAILURE_CHARS: usize = 300;

fn failure_text(error: &anyhow::Error) -> String {
    format!("{error:#}")
        .chars()
        .take(MAX_FAILURE_CHARS)
        .collect()
}

enum Backend {
    /// whisper.cpp in this process: on the processor, or on a Mac's graphics processor (Metal).
    InProcess {
        context: WhisperContext,
        device: &'static str,
    },
    /// The Vulkan helper, on the graphics processor it chose.
    Helper(Box<gpu::Worker>),
}

/// Where a session asks for its model to run.
pub(super) struct Placement<'a> {
    /// Metal, in this process (macOS).
    pub(super) metal: bool,
    /// The Vulkan helper to try first (Windows).
    pub(super) helper: Option<&'a Path>,
    /// Let the helper choose a software renderer: the tests' only graphics processor.
    pub(super) allow_software: bool,
    /// The session's discard token: ends a load that hangs in the helper, rather than holding
    /// the session's start (and its lifecycle lock) for the helper's whole timeout.
    pub(super) abort: &'a CancellationToken,
}

/// Load `id` for a session. `graphics` is the operator's choice to use the graphics processor
/// where there is one; `abort`, the session's discard token, ends a load stuck in the helper.
pub fn load(
    app: &AppHandle,
    id: ModelId,
    graphics: bool,
    abort: &CancellationToken,
) -> Result<Arc<LoadedModel>> {
    let lease = app
        .state::<ModelManager>()
        .lease(&models::directory(app)?, id)?;
    let helper = graphics.then(gpu::helper_path).flatten();
    load_lease(
        lease,
        vad_path(app).as_deref(),
        &Placement {
            metal: graphics && cfg!(target_os = "macos"),
            helper: helper.as_deref(),
            allow_software: false,
            abort,
        },
    )
}

/// The bundled speech-detection model, where the installer put it.
fn vad_path(app: &AppHandle) -> Option<PathBuf> {
    vad::bundled(app)
}

pub(super) fn load_lease(
    lease: ModelLease,
    vad: Option<&Path>,
    placement: &Placement<'_>,
) -> Result<Arc<LoadedModel>> {
    // First native call of a session: refuse here, not with an illegal instruction inside it.
    // Speech detection and the fallback run on this processor even when the helper decodes.
    cpu::ensure_supported()?;
    let mut gpu_failure = None;
    let helper = placement.helper.and_then(|program| {
        // The helper checks the file against the pinned digest before it loads it: once it
        // has, a fallback to the processor later in the session need not hash it again.
        let before = lease.stamp().ok();
        gpu::Worker::start(program, &lease, placement.allow_software, placement.abort)
            .inspect(|_| {
                if let Some(before) = before {
                    lease.verified(before);
                }
            })
            .inspect_err(|error| {
                tracing::info!("Whisper stays on the processor: {error:#}");
                gpu_failure = Some(failure_text(error));
            })
            .ok()
    });
    ensure!(
        !placement.abort.is_cancelled(),
        "Model loading was cancelled"
    );
    let backend = match helper {
        Some(worker) => Backend::Helper(Box::new(worker)),
        None => in_process(&lease, placement.metal)?,
    };
    let model = LoadedModel {
        inference: Mutex::new(backend),
        vad: vad.and_then(vad::Vad::load),
        lease,
        gpu_failure: Mutex::new(gpu_failure),
        engine_changed: AtomicBool::new(false),
    };
    tracing::info!(device = %model.device(), "Whisper model loaded");
    Ok(Arc::new(model))
}

/// The leased model in this process, with Metal when `metal`.
fn in_process(lease: &ModelLease, metal: bool) -> Result<Backend> {
    // Read through Rust's Unicode-aware filesystem API (Windows usernames may contain
    // non-ASCII characters), then pass the exact verified bytes to the native loader.
    let bytes = lease.read_verified()?;
    let mut params = WhisperContextParameters::default();
    params.use_gpu(metal);
    if metal {
        // Attention computed in one pass, as whisper.cpp's own tools do on a graphics processor.
        params.flash_attn(true);
    }
    let context = WhisperContext::new_from_buffer_with_params(&bytes, params)
        .context("Could not load Whisper model")?;
    Ok(Backend::InProcess {
        context,
        device: if metal { "metal" } else { "cpu" },
    })
}

/// What one window decoded to: its segments, and the language whisper.cpp decoded it in.
pub(super) struct Decoded {
    pub(super) segments: Vec<RawSegment>,
    pub(super) language: i32,
}

impl LoadedModel {
    /// Where the model runs now: `cpu`, `metal`, or the graphics processor the helper chose.
    pub fn device(&self) -> String {
        match &*lock(&self.inference) {
            Backend::InProcess { device, .. } => (*device).to_owned(),
            Backend::Helper(worker) => format!("vulkan: {}", worker.device),
        }
    }

    /// Where the windows are decoded now, and why not on the graphics processor if it failed.
    pub fn engine(&self) -> Engine {
        let (backend, device) = match &*lock(&self.inference) {
            Backend::InProcess { device, .. } => (*device, None),
            Backend::Helper(worker) => ("vulkan", Some(worker.device.clone())),
        };
        Engine {
            backend,
            device,
            gpu_failure: lock(&self.gpu_failure).clone(),
        }
    }

    pub(super) fn text_context(&self) -> i32 {
        match &*lock(&self.inference) {
            Backend::InProcess { context, .. } => context.n_text_ctx(),
            Backend::Helper(worker) => worker.text_context,
        }
    }

    pub(super) fn tokenize(&self, text: &str) -> Option<Vec<i32>> {
        match &mut *lock(&self.inference) {
            // Every token stands for at least one byte, so the text's length is room enough:
            // whisper-rs mishandles whisper.cpp's "needs more room" answer, so it must never
            // get one.
            Backend::InProcess { context, .. } => context.tokenize(text, text.len() + 1).ok(),
            Backend::Helper(worker) => worker.tokenize(text),
        }
    }

    /// The part of a window that holds speech; all of it without speech detection.
    pub(super) fn speech(&self, samples: &[f32]) -> Option<std::ops::Range<usize>> {
        match &self.vad {
            Some(vad) => vad.speech(samples),
            None => Some(0..samples.len()),
        }
    }

    /// Decode one window. `state` is the source's own whisper.cpp state, made on first use in
    /// this process. `None` when the operator discarded the remaining audio meanwhile. A helper
    /// that fails (one window failing twice, or the helper itself failing; see
    /// `gpu::Worker::decode`) is replaced, for the rest of the session, by the model on the
    /// processor, and the window is decoded there.
    pub(super) fn decode(
        &self,
        state: &mut Option<WhisperState>,
        decode: &engine::Decode,
        samples: &[f32],
        abort: &CancellationToken,
    ) -> Result<Option<Decoded>> {
        let mut backend = lock(&self.inference);
        if let Backend::Helper(worker) = &mut *backend {
            match judge(worker.decode(decode, samples, abort), abort) {
                HelperVerdict::Decoded((segments, language)) => {
                    return Ok(Some(Decoded { segments, language }));
                }
                HelperVerdict::Discarded => return Ok(None),
                HelperVerdict::FallBack(error) => {
                    tracing::warn!(
                        "Whisper's graphics processor failed; carrying on on the processor: {error:#}"
                    );
                    move_to_processor(
                        &mut *backend,
                        &error,
                        &self.gpu_failure,
                        &self.engine_changed,
                        |backend| {
                            if let Backend::Helper(worker) = backend {
                                worker.end();
                            }
                        },
                        || in_process(&self.lease, false),
                    )?;
                }
            }
        }
        // After a move, the same window, decoded on the processor.
        let Backend::InProcess { context, .. } = &*backend else {
            anyhow::bail!("Whisper has no model to decode with");
        };
        let state = match state {
            Some(state) => state,
            None => state.insert(context.create_state()?),
        };
        let mut params = engine::params(decode);
        // whisper-rs 0.16's closure helper has an incorrect user-data cast and leaks its
        // allocation. Use the C callback with a borrowed, thread-safe token instead.
        // SAFETY: full() is synchronous; abort outlives the entire call, and the callback
        // only reads CancellationToken. No pointer escapes this invocation.
        unsafe {
            params.set_abort_callback(Some(abort_requested));
            params.set_abort_callback_user_data(std::ptr::from_ref(abort).cast_mut().cast());
        }
        match state.full(params, samples) {
            Ok(()) => Ok(Some(Decoded {
                segments: engine::segments(state),
                language: state.full_lang_id_from_state(),
            })),
            Err(_) if abort.is_cancelled() => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

/// What the helper's attempt at a window means for the session.
#[derive(Debug)]
enum HelperVerdict<T> {
    /// Decoded: the session stays on the graphics processor.
    Decoded(T),
    /// The operator discarded the remaining audio meanwhile: no caption, and no move, since
    /// the helper failed only because it was told to stop.
    Discarded,
    /// The helper failed: the session moves to the processor and decodes this window there.
    FallBack(anyhow::Error),
}

fn judge<T>(attempt: Result<T>, abort: &CancellationToken) -> HelperVerdict<T> {
    match attempt {
        Ok(decoded) => HelperVerdict::Decoded(decoded),
        Err(_) if abort.is_cancelled() => HelperVerdict::Discarded,
        Err(error) => HelperVerdict::FallBack(error),
    }
}

/// Replace a failed helper in `backend` with the model on the processor, in this order: the
/// failure is recorded (so the operator learns why even if the processor cannot load either),
/// the helper is ended (it holds its own copy of the model, and the graphics processor's
/// memory, while the processor loads another), the processor model is loaded into its place,
/// and only then are the sources told the engine changed. Generic over the backend, so the
/// order is tested without a model.
fn move_to_processor<B>(
    backend: &mut B,
    error: &anyhow::Error,
    gpu_failure: &Mutex<Option<String>>,
    engine_changed: &AtomicBool,
    end_helper: impl FnOnce(&mut B),
    load: impl FnOnce() -> Result<B>,
) -> Result<()> {
    *lock(gpu_failure) = Some(failure_text(error));
    end_helper(backend);
    *backend = load()?;
    engine_changed.store(true, Ordering::Relaxed);
    Ok(())
}

unsafe extern "C" fn abort_requested(data: *mut std::ffi::c_void) -> bool {
    // SAFETY: installed only above, with a live CancellationToken for the full() duration.
    unsafe { &*data.cast::<CancellationToken>() }.is_cancelled()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    /// A backend as the move sees it, without a model.
    #[derive(Debug, PartialEq)]
    enum Fake {
        Helper { ended: bool },
        Processor,
    }

    #[test]
    fn a_window_the_helper_decodes_stays_on_it_and_a_discard_moves_nothing() {
        let abort = CancellationToken::new();
        assert!(matches!(
            judge(Ok::<_, anyhow::Error>(7), &abort),
            HelperVerdict::Decoded(7)
        ));
        assert!(matches!(
            judge::<()>(Err(anyhow::anyhow!("stopped answering")), &abort),
            HelperVerdict::FallBack(_)
        ));
        abort.cancel();
        // A helper ended by Discard has not failed: no move, no failure shown.
        assert!(matches!(
            judge::<()>(Err(anyhow::anyhow!("Discarded")), &abort),
            HelperVerdict::Discarded
        ));
    }

    /// The mid-session move: the failure is recorded, the helper ended before the processor
    /// loads its own copy of the model, the backend replaced, and the engine change flagged
    /// for the sources to report.
    #[test]
    fn a_failed_helper_is_ended_then_replaced_by_the_processor() {
        let gpu_failure = Mutex::new(None);
        let engine_changed = AtomicBool::new(false);
        let steps = RefCell::new(Vec::new());
        let mut backend = Fake::Helper { ended: false };
        let error = anyhow::anyhow!("The graphics processor helper stopped answering")
            .context("ggml_vulkan: error ErrorDeviceLost");
        move_to_processor(
            &mut backend,
            &error,
            &gpu_failure,
            &engine_changed,
            |backend| {
                assert!(
                    lock(&gpu_failure).is_some(),
                    "recorded before anything else"
                );
                assert_eq!(*backend, Fake::Helper { ended: false });
                *backend = Fake::Helper { ended: true };
                steps.borrow_mut().push("end helper");
            },
            || {
                assert!(!engine_changed.load(Ordering::Relaxed));
                steps.borrow_mut().push("load processor");
                Ok(Fake::Processor)
            },
        )
        .unwrap();
        assert_eq!(*steps.borrow(), ["end helper", "load processor"]);
        assert_eq!(backend, Fake::Processor);
        assert!(engine_changed.load(Ordering::Relaxed));
        assert_eq!(
            lock(&gpu_failure).as_deref(),
            Some(
                "ggml_vulkan: error ErrorDeviceLost: The graphics processor helper stopped answering"
            )
        );
    }

    /// When the processor cannot load the model either, the session's error is that load's,
    /// the helper's failure is still recorded, and no engine change is announced.
    #[test]
    fn a_processor_that_cannot_load_leaves_the_failure_recorded_and_no_change_announced() {
        let gpu_failure = Mutex::new(None);
        let engine_changed = AtomicBool::new(false);
        let mut backend = Fake::Helper { ended: false };
        let error = move_to_processor(
            &mut backend,
            &anyhow::anyhow!("helper died"),
            &gpu_failure,
            &engine_changed,
            |backend| *backend = Fake::Helper { ended: true },
            || Err(anyhow::anyhow!("Could not load Whisper model")),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "Could not load Whisper model");
        assert_eq!(backend, Fake::Helper { ended: true });
        assert!(!engine_changed.load(Ordering::Relaxed));
        assert_eq!(lock(&gpu_failure).as_deref(), Some("helper died"));
    }

    #[test]
    fn a_long_failure_is_cut_to_its_first_characters() {
        let error = anyhow::anyhow!("é".repeat(MAX_FAILURE_CHARS * 2));
        assert_eq!(failure_text(&error).chars().count(), MAX_FAILURE_CHARS);
    }
}
