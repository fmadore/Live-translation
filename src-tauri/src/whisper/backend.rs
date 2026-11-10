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
    Helper(gpu::Worker),
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
    app.path()
        .resolve(vad::RESOURCE, tauri::path::BaseDirectory::Resource)
        .ok()
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
        Some(worker) => Backend::Helper(worker),
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
            match worker.decode(decode, samples, abort) {
                Ok((segments, language)) => return Ok(Some(Decoded { segments, language })),
                Err(_) if abort.is_cancelled() => return Ok(None),
                Err(error) => {
                    tracing::warn!(
                        "Whisper's graphics processor failed; carrying on on the processor: {error:#}"
                    );
                    *lock(&self.gpu_failure) = Some(failure_text(&error));
                    // End the helper first: it holds its own copy of the model, and the
                    // graphics processor's memory, while the processor loads another.
                    worker.end();
                    *backend = in_process(&self.lease, false)?;
                    self.engine_changed.store(true, Ordering::Relaxed);
                }
            }
        }
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

unsafe extern "C" fn abort_requested(data: *mut std::ffi::c_void) -> bool {
    // SAFETY: installed only above, with a live CancellationToken for the full() duration.
    unsafe { &*data.cast::<CancellationToken>() }.is_cancelled()
}
