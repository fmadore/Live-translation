//! Multilingual local transcription, or translation into English with Whisper's translate task.
//! Capture, disk ingestion and inference run independently; normal Stop closes input then
//! drains every accepted frame.
//!
//! Where the windows are decoded: on Windows, on the graphics processor through the Vulkan
//! helper (`gpu.rs`) when one is installed and finds a usable device; on a Mac, on the graphics
//! processor through Metal, in the app; otherwise, and whenever the graphics processor fails,
//! on the processor. Silero's speech detection (`vad.rs`) decides first whether a window is
//! worth decoding at all.
pub mod cpu;
// Shared word for word with the Vulkan helper, so both decode a window the same way. Some of
// it (the helper's half of the messages) is only used there.
#[allow(dead_code)]
#[path = "../../whisper-vulkan/src/engine.rs"]
mod engine;
mod gpu;
mod language;
pub mod models;
mod prompt;
mod reconcile;
mod segment;
pub mod spool;
mod vad;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::Ordering};
use std::time::Duration;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use whisper_rs::{WhisperContext, WhisperContextParameters, WhisperState};

use crate::errors::{AppError, id};
use crate::realtime::Events;
use crate::sync::lock;
use crate::types::{Caption, Origin, SessionState, StatusUpdate, events};
use engine::RawSegment;
use language::Languages;
use models::{ModelId, ModelLease, ModelManager};
use prompt::Prompt;
use reconcile::Reconciler;
use segment::{Segmenter, Window};
use spool::{Next, Spool, TimedChunk};

/// Input quiet for this long releases whatever audio is held, so the last words before Pause,
/// or before a call goes silent, are captioned now rather than when audio resumes.
const IDLE: Duration = Duration::from_secs(1);

/// A backlog beyond this switches to long windows; see `segment::LONG_WINDOW`.
const LONG_WINDOW_BACKLOG_MS: u64 = 30_000;

pub struct LoadedModel {
    /// Where windows are decoded, one at a time across sources. With Both sources, two
    /// concurrent passes would each start a full set of ggml workers, which spin while they
    /// wait and leave capture and the interface fighting for what is left; taking turns
    /// finishes both sooner. The graphics processor helper takes one window at a time anyway.
    inference: Mutex<Backend>,
    vad: Option<vad::Vad>,
    lease: ModelLease,
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
struct Placement<'a> {
    /// Metal, in this process (macOS).
    metal: bool,
    /// The Vulkan helper to try first (Windows).
    helper: Option<&'a Path>,
    /// Let the helper choose a software renderer: the tests' only graphics processor.
    allow_software: bool,
}

/// Load `id` for a session. `graphics` is the operator's choice to use the graphics processor
/// where there is one.
pub fn load(app: &AppHandle, id: ModelId, graphics: bool) -> Result<Arc<LoadedModel>> {
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
        },
    )
}

/// The bundled speech-detection model, where the installer put it.
fn vad_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .resolve(vad::RESOURCE, tauri::path::BaseDirectory::Resource)
        .ok()
}

fn load_lease(
    lease: ModelLease,
    vad: Option<&Path>,
    placement: &Placement<'_>,
) -> Result<Arc<LoadedModel>> {
    // First native call of a session: refuse here, not with an illegal instruction inside it.
    // Speech detection and the fallback run on this processor even when the helper decodes.
    cpu::ensure_supported()?;
    let helper = placement.helper.and_then(|program| {
        gpu::Worker::start(program, &lease, placement.allow_software)
            .inspect_err(|error| {
                tracing::info!("Whisper stays on the processor: {error:#}");
            })
            .ok()
    });
    let backend = match helper {
        Some(worker) => Backend::Helper(worker),
        None => in_process(&lease, placement.metal)?,
    };
    let model = LoadedModel {
        inference: Mutex::new(backend),
        vad: vad.and_then(vad::Vad::load),
        lease,
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
struct Decoded {
    segments: Vec<RawSegment>,
    language: i32,
}

impl LoadedModel {
    /// Where the model runs now: `cpu`, `metal`, or the graphics processor the helper chose.
    pub fn device(&self) -> String {
        match &*lock(&self.inference) {
            Backend::InProcess { device, .. } => (*device).to_owned(),
            Backend::Helper(worker) => format!("vulkan: {}", worker.device),
        }
    }

    fn text_context(&self) -> i32 {
        match &*lock(&self.inference) {
            Backend::InProcess { context, .. } => context.n_text_ctx(),
            Backend::Helper(worker) => worker.text_context,
        }
    }

    fn tokenize(&self, text: &str) -> Option<Vec<i32>> {
        match &mut *lock(&self.inference) {
            // Every token stands for at least one byte, so the text's length is room enough:
            // whisper-rs mishandles whisper.cpp's "needs more room" answer, so it must never
            // get one.
            Backend::InProcess { context, .. } => context.tokenize(text, text.len() + 1).ok(),
            Backend::Helper(worker) => worker.tokenize(text),
        }
    }

    /// The part of a window that holds speech; all of it without speech detection.
    fn speech(&self, samples: &[f32]) -> Option<std::ops::Range<usize>> {
        match &self.vad {
            Some(vad) => vad.speech(samples),
            None => Some(0..samples.len()),
        }
    }

    /// Decode one window. `state` is the source's own whisper.cpp state, made on first use in
    /// this process. `None` when the operator discarded the remaining audio meanwhile. A helper
    /// that fails is replaced, for the rest of the session, by the model on the processor, and
    /// the window is decoded there.
    fn decode(
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
                    *backend = in_process(&self.lease, false)?;
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

/// ggml's workers busy-wait between operations, so a thread per hardware thread starves
/// capture and the interface, and beyond about eight these small models stop getting faster.
/// x64 processors are assumed to have SMT: half the logical processors is one thread per core,
/// and the sibling hardware threads absorb capture and the interface. Windows-on-ARM has no
/// SMT, so one core is left free instead.
fn inference_threads(logical: usize) -> i32 {
    let threads = if cfg!(target_arch = "x86_64") {
        logical / 2
    } else {
        logical.saturating_sub(1)
    };
    i32::try_from(threads).unwrap_or(i32::MAX).clamp(1, 8)
}

pub fn validate_language(language: Option<&str>) -> Result<()> {
    let Some(language) = language else {
        return Ok(());
    };
    // These quantized multilingual tiny/base/small models have the original 99 languages.
    // Cantonese's separate token belongs to large-v3 and is deliberately not advertised.
    ensure!(!language.contains('\0'), "Invalid spoken language");
    ensure!(
        whisper_rs::get_lang_id(language).is_some_and(|id| id < 99),
        "Unsupported Whisper spoken language"
    );
    Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    origin: Origin,
    pending_ms: u64,
    finalizing: bool,
}

fn progress(events: &impl LocalEvents, origin: Origin, spool: &Spool) {
    events.progress(Progress {
        origin,
        pending_ms: spool.pending_ms(),
        finalizing: spool.finalizing.load(Ordering::Relaxed),
    });
}

/// Where a local source's captions, statuses and backlog progress go: the app in production,
/// a recorder in the tests, so the pipeline's endings are tested without Tauri or a model.
trait LocalEvents: Events + Clone + Send + 'static {
    fn progress(&self, progress: Progress);
}

impl LocalEvents for AppHandle {
    fn progress(&self, progress: Progress) {
        let _ = self.emit(events::WHISPER_PROGRESS, progress);
    }
}

pub struct LocalSession {
    pub app: AppHandle,
    pub model: Arc<LoadedModel>,
    pub language: Option<String>,
    /// Write English, whatever is spoken, rather than what was said in its own language.
    pub translate: bool,
    /// The session's names and terms, given to every window as its prompt.
    pub glossary: Vec<String>,
    pub origin: Origin,
    pub input: Receiver<TimedChunk>,
    pub spool: Arc<Spool>,
    pub capture_cancel: CancellationToken,
    pub abort: CancellationToken,
    pub failure: Arc<Mutex<Option<AppError>>>,
}

pub async fn run(config: LocalSession) {
    let LocalSession {
        app,
        model,
        language,
        translate,
        glossary,
        origin,
        input,
        spool,
        capture_cancel,
        abort,
        failure,
    } = config;
    let pipeline = Pipeline {
        events: app,
        origin,
        input,
        spool,
        capture_cancel,
        abort,
        failure,
    };
    drive(pipeline, move |spool, abort, emit, report| {
        transcribe(
            &model,
            &Task {
                language: language.as_deref(),
                translate,
                glossary: &glossary,
            },
            spool,
            abort,
            emit,
            report,
        )
    })
    .await;
}

/// A local source without its model: everything `drive` needs besides the inference itself.
struct Pipeline<E> {
    events: E,
    origin: Origin,
    input: Receiver<TimedChunk>,
    spool: Arc<Spool>,
    capture_cancel: CancellationToken,
    abort: CancellationToken,
    failure: Arc<Mutex<Option<AppError>>>,
}

/// One caption from the inference: its text, start and end in session milliseconds, and turn.
type Emit<'a> = &'a mut dyn FnMut(&str, u64, u64, u64);
/// Called after each window, to report the backlog.
type Report<'a> = &'a mut dyn FnMut();

/// Write the input to the spool on one blocking worker, run `infer` over it on another, and
/// report how the source ended once both have: Idle after a full drain, or Error with why —
/// a failed write, a capture failure recorded on the spool, a failed or panicked inference, or
/// the operator discarding what was left. `infer` turns the spool into captions as `transcribe`
/// does, reading it until it ends or the abort token fires; the tests stand in for Whisper
/// with it.
async fn drive<E: LocalEvents>(
    pipeline: Pipeline<E>,
    infer: impl FnOnce(&Spool, &CancellationToken, Emit<'_>, Report<'_>) -> Result<()> + Send + 'static,
) {
    let Pipeline {
        events: app,
        origin,
        mut input,
        spool,
        capture_cancel,
        abort,
        failure,
    } = pipeline;
    app.status(StatusUpdate::source(origin, SessionState::Running, None));
    let ingest_spool = spool.clone();
    let ingest_app = app.clone();
    let ingest_cancel = capture_cancel.clone();
    let ingestion = tauri::async_runtime::spawn_blocking(move || -> Result<()> {
        let _cancel = ingest_cancel.clone().drop_guard();
        // Always signal EOF, including on an I/O failure or a panic.
        struct Finish(Arc<Spool>);
        impl Drop for Finish {
            fn drop(&mut self) {
                self.0.finish();
            }
        }
        let _finish = Finish(ingest_spool.clone());
        let result = (|| {
            let mut count = 0usize;
            while let Some(frame) = input.blocking_recv() {
                ingest_spool.push(&frame)?;
                count += 1;
                if count.is_multiple_of(10) {
                    progress(&ingest_app, origin, &ingest_spool);
                }
            }
            ensure!(
                !ingest_spool.overflowed.load(Ordering::Relaxed),
                "Audio capture outran the local disk buffer; the transcript may be incomplete"
            );
            if let Some(error) = lock(&ingest_spool.capture_error).as_ref() {
                anyhow::bail!("{error}");
            }
            Ok(())
        })();
        if result.is_err() {
            ingest_cancel.cancel();
        }
        ingest_spool.finish();
        progress(&ingest_app, origin, &ingest_spool);
        result
    });
    let worker_spool = spool.clone();
    let worker_app = app.clone();
    let worker_abort = abort.clone();
    let worker_cancel = capture_cancel.clone();
    let inference = tauri::async_runtime::spawn_blocking(move || {
        // Unwinding must also stop capture, otherwise the other joined task waits forever.
        let _cancel = worker_cancel.drop_guard();
        infer(
            &worker_spool,
            &worker_abort,
            &mut |text, start, end, turn| {
                worker_app.caption(Caption {
                    turn_id: turn,
                    text,
                    // Translating as well: the translate task returns English and no
                    // transcription of what was said, so there is no original to show.
                    source_text: "",
                    final_: true,
                    origin,
                    lane: 0,
                    start_ms: start,
                    end_ms: end,
                });
            },
            &mut || progress(&worker_app, origin, &worker_spool),
        )
    });
    let (ingested, inferred) = tokio::join!(ingestion, inference);
    let result = ingested
        .context("Local audio writer stopped unexpectedly")
        .and_then(|r| r)
        .and_then(|_| {
            inferred
                .context("Whisper worker stopped unexpectedly")
                .and_then(|r| r)
        });
    capture_cancel.cancel();
    progress(&app, origin, &spool);
    let result = result.and_then(|_| {
        ensure!(
            !abort.is_cancelled(),
            "Remaining audio was discarded; the transcript is incomplete"
        );
        Ok(())
    });
    if let Err(error) = result {
        *lock(&failure) = Some(AppError::with(id::WHISPER_SESSION, &error));
        app.status(StatusUpdate::source(
            origin,
            SessionState::Error,
            Some(AppError::with(id::WHISPER_SESSION, &error)),
        ));
    } else {
        app.status(StatusUpdate::source(origin, SessionState::Idle, None));
    }
}

/// What a source's windows are decoded as: the spoken language, or `None` to detect it; whether
/// to translate into English; and the session's names and terms, for the prompt.
struct Task<'a> {
    language: Option<&'a str>,
    translate: bool,
    glossary: &'a [String],
}

fn transcribe(
    model: &LoadedModel,
    task: &Task<'_>,
    spool: &Spool,
    abort: &CancellationToken,
    mut emit: impl FnMut(&str, u64, u64, u64),
    mut report: impl FnMut(),
) -> Result<()> {
    let mut state = None;
    let prompt = glossary_prompt(model, task.glossary);
    let mut reconciler = Reconciler::default();
    let mut languages = Languages::new(task.language);
    let threads = inference_threads(std::thread::available_parallelism().map_or(2, usize::from));
    let process = |window: Window| -> Result<()> {
        if abort.is_cancelled() {
            return Ok(());
        }
        // Silence by the segmenter's measure, or no speech by Silero's: nothing to decode.
        if let Some(speech) = window
            .speech
            .then(|| model.speech(&window.samples))
            .flatten()
        {
            let start_ms = window.start_ms + speech.start as u64 / 16;
            let mut samples = window.samples[speech].to_vec();
            let duration_ms = samples.len() as u64 / 16;
            let spoken = languages.for_window(duration_ms);
            let decode = engine::Decode {
                language: spoken.map(str::to_owned),
                translate: task.translate,
                threads,
                prompt: prompt.tokens.clone(),
            };
            // Whisper needs enough samples for its analysis window, including a final
            // phrase shorter than a second. Padding affects inference only, not timestamps.
            samples.resize(samples.len().max(16_000), 0.0);
            let Some(decoded) = model.decode(&mut state, &decode, &samples, abort)? else {
                return Ok(());
            };
            let mut captioned = false;
            for segment in &decoded.segments {
                if prompt.is_echo(&segment.text) {
                    continue;
                }
                if let Some(line) = reconciler.accept(start_ms, duration_ms, segment) {
                    emit(&line.text, line.start_ms, line.end_ms, line.turn);
                    captioned = true;
                }
            }
            let detected = whisper_rs::get_lang_str(decoded.language)
                .filter(|&code| validate_language(Some(code)).is_ok());
            languages.observe(duration_ms, spoken, captioned, detected);
        }
        spool.complete(window.consumed);
        report();
        Ok(())
    };
    pump(spool, abort, IDLE, process)
}

/// The session's names and terms as tokens for every window's prompt: as many as fit, from the
/// top of the list. whisper.cpp gives a prompt at most half the text context, less the token
/// that opens it — 223 tokens with every Whisper model, a few dozen names.
fn glossary_prompt(model: &LoadedModel, glossary: &[String]) -> Prompt {
    let budget = usize::try_from(model.text_context() / 2 - 1).unwrap_or(0);
    let prompt = Prompt::fit(glossary, budget, |text| model.tokenize(text));
    if prompt.len() < glossary.len() {
        tracing::info!(
            kept = prompt.len(),
            listed = glossary.len(),
            "Whisper's prompt holds only the first names and terms"
        );
    }
    prompt
}

/// Feed spooled audio through the segmenter to `process` until input ends or is discarded.
fn pump(
    spool: &Spool,
    abort: &CancellationToken,
    idle: Duration,
    mut process: impl FnMut(Window) -> Result<()>,
) -> Result<()> {
    let mut segmenter = Segmenter::default();
    while !abort.is_cancelled() {
        segmenter.set_long_windows(spool.pending_ms() > LONG_WINDOW_BACKLOG_MS);
        match spool.next_timeout(idle)? {
            Next::Chunk(frame) => {
                for window in segmenter.push(&frame) {
                    process(window)?;
                }
            }
            Next::Idle => {
                if let Some(window) = segmenter.finish() {
                    process(window)?;
                }
            }
            Next::End => break,
        }
    }
    if !abort.is_cancelled() {
        if let Some(window) = segmenter.finish() {
            process(window)?;
        }
    }
    Ok(())
}

unsafe extern "C" fn abort_requested(data: *mut std::ffi::c_void) -> bool {
    // SAFETY: installed only above, with a live CancellationToken for the full() duration.
    unsafe { &*data.cast::<CancellationToken>() }.is_cancelled()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn speech(start_ms: u64) -> TimedChunk {
        TimedChunk {
            start_ms,
            pcm: 100i16.to_le_bytes().repeat(1600),
        }
    }

    #[derive(Debug, PartialEq)]
    enum Recorded {
        Status(SessionState, Option<&'static str>),
        Caption(String),
        Progress(u64),
    }

    /// What a local source reports, in order, in place of the app's events.
    #[derive(Clone, Default)]
    struct Recorder(Arc<Mutex<Vec<Recorded>>>);

    impl Recorder {
        fn push(&self, recorded: Recorded) {
            self.0.lock().unwrap().push(recorded);
        }

        fn statuses(&self) -> Vec<Recorded> {
            let recorded = self.0.lock().unwrap();
            let statuses = recorded
                .iter()
                .filter(|r| matches!(r, Recorded::Status(..)));
            statuses
                .map(|r| match r {
                    Recorded::Status(state, id) => Recorded::Status(*state, *id),
                    _ => unreachable!(),
                })
                .collect()
        }

        fn captions(&self) -> usize {
            let recorded = self.0.lock().unwrap();
            recorded
                .iter()
                .filter(|r| matches!(r, Recorded::Caption(_)))
                .count()
        }

        fn last_progress(&self) -> Option<u64> {
            let recorded = self.0.lock().unwrap();
            recorded.iter().rev().find_map(|r| match r {
                Recorded::Progress(pending_ms) => Some(*pending_ms),
                _ => None,
            })
        }
    }

    impl Events for Recorder {
        fn caption(&self, caption: Caption<'_>) {
            self.push(Recorded::Caption(caption.text.to_string()));
        }

        fn status(&self, update: StatusUpdate) {
            let id = update.message.map(|message| message.id);
            self.push(Recorded::Status(update.state, id));
        }
    }

    impl LocalEvents for Recorder {
        fn progress(&self, progress: Progress) {
            self.push(Recorded::Progress(progress.pending_ms));
        }
    }

    /// A local source with nothing captured yet, and the producer end of its input.
    fn pipeline() -> (Pipeline<Recorder>, tokio::sync::mpsc::Sender<TimedChunk>) {
        let (input_tx, input) = tokio::sync::mpsc::channel(300);
        let pipeline = Pipeline {
            events: Recorder::default(),
            origin: Origin::Microphone,
            input,
            spool: Arc::new(Spool::new().unwrap()),
            capture_cancel: CancellationToken::new(),
            abort: CancellationToken::new(),
            failure: Arc::new(Mutex::new(None)),
        };
        (pipeline, input_tx)
    }

    /// Whisper's stand-in: one caption per speech window, through the production pump.
    fn echo(
        spool: &Spool,
        abort: &CancellationToken,
        emit: Emit<'_>,
        report: Report<'_>,
    ) -> Result<()> {
        pump(spool, abort, Duration::from_millis(10), |window| {
            spool.complete(window.consumed);
            if window.speech {
                let end_ms = window.start_ms + window.samples.len() as u64 / 16;
                emit("words", window.start_ms, end_ms, 0);
            }
            report();
            Ok(())
        })
    }

    fn whisper_error(failure: &Mutex<Option<AppError>>) -> String {
        let failure = failure.lock().unwrap();
        let error = failure.as_ref().expect("the session recorded no failure");
        assert_eq!(error.id, id::WHISPER_SESSION);
        error.detail.clone().unwrap_or_default()
    }

    const RUNNING: Recorded = Recorded::Status(SessionState::Running, None);
    const IDLE: Recorded = Recorded::Status(SessionState::Idle, None);
    const FAILED: Recorded = Recorded::Status(SessionState::Error, Some(id::WHISPER_SESSION));

    /// Stop closes the input; every frame accepted before it is still captioned, and only then
    /// is the source Idle.
    #[tokio::test]
    async fn stop_drains_every_accepted_frame_then_reports_idle() {
        let (pipeline, input) = pipeline();
        let (events, spool, failure) = (
            pipeline.events.clone(),
            pipeline.spool.clone(),
            pipeline.failure.clone(),
        );
        for i in 0..20 {
            input.send(speech(i * 100)).await.unwrap();
        }
        drop(input);
        drive(pipeline, echo).await;

        assert_eq!(events.statuses(), [RUNNING, IDLE]);
        assert!(events.captions() > 0);
        assert_eq!(spool.pending_ms(), 0);
        assert_eq!(events.last_progress(), Some(0));
        assert!(failure.lock().unwrap().is_none());
    }

    /// A capture failure recorded on the spool — a capture-thread panic, say (B8) — reaches the
    /// operator once what was captured before it has been captioned.
    #[tokio::test]
    async fn a_capture_failure_is_reported_after_the_audio_before_it() {
        let (pipeline, input) = pipeline();
        let (events, spool, failure) = (
            pipeline.events.clone(),
            pipeline.spool.clone(),
            pipeline.failure.clone(),
        );
        for i in 0..20 {
            input.send(speech(i * 100)).await.unwrap();
        }
        *spool.capture_error.lock().unwrap() =
            Some("the capture thread panicked: device lost".into());
        drop(input);
        drive(pipeline, echo).await;

        assert_eq!(events.statuses(), [RUNNING, FAILED]);
        assert!(
            events.captions() > 0,
            "the audio before the failure was dropped"
        );
        assert_eq!(
            whisper_error(&failure),
            "the capture thread panicked: device lost"
        );
    }

    /// A panicking inference worker must still stop capture, or the writer, waiting for input
    /// that never ends, would hold Stop forever; and the source must say it failed.
    #[tokio::test]
    async fn an_inference_panic_stops_capture_and_is_reported() {
        let (pipeline, input) = pipeline();
        let (events, failure, capture) = (
            pipeline.events.clone(),
            pipeline.failure.clone(),
            pipeline.capture_cancel.clone(),
        );
        // As capture does: its producer end goes once its token is cancelled.
        let capture_stopped = capture.clone();
        tokio::spawn(async move {
            capture_stopped.cancelled().await;
            drop(input);
        });
        tokio::time::timeout(
            Duration::from_secs(10),
            drive(pipeline, |_, _, _, _| panic!("ggml aborted")),
        )
        .await
        .expect("the writer was left waiting for input");

        assert!(capture.is_cancelled());
        assert_eq!(events.statuses(), [RUNNING, FAILED]);
        let detail = whisper_error(&failure);
        assert!(
            detail.starts_with("Whisper worker stopped unexpectedly"),
            "{detail}"
        );
    }

    /// The operator discarding the backlog while Stop waits ends the source in an error that
    /// says the transcript is incomplete, not in a clean Idle.
    #[tokio::test]
    async fn discarding_the_backlog_is_reported_as_an_incomplete_transcript() {
        let (pipeline, input) = pipeline();
        let (events, failure) = (pipeline.events.clone(), pipeline.failure.clone());
        input.send(speech(0)).await.unwrap();
        drop(input);
        pipeline.abort.cancel();
        drive(pipeline, echo).await;

        assert_eq!(events.statuses(), [RUNNING, FAILED]);
        assert_eq!(events.captions(), 0);
        assert_eq!(
            whisper_error(&failure),
            "Remaining audio was discarded; the transcript is incomplete"
        );
    }

    #[test]
    fn inference_threads_leave_room_for_capture_and_stop_at_eight() {
        if cfg!(target_arch = "x86_64") {
            assert_eq!(
                [1, 2, 4, 8, 12, 16, 32].map(inference_threads),
                [1, 1, 2, 4, 6, 8, 8]
            );
        } else {
            assert_eq!(
                [1, 2, 4, 8, 12, 16].map(inference_threads),
                [1, 1, 3, 7, 8, 8]
            );
        }
    }

    #[test]
    fn the_task_reaches_whisper_with_the_spoken_language_still_set() {
        // whisper-rs exposes no getters; its Debug output shows the native parameters.
        for translate in [false, true] {
            let params = format!("{:?}", engine::decode_params(Some("fr"), translate, 4, &[]));
            assert!(
                params.contains(&format!("translate: {translate},")),
                "{params}"
            );
            assert!(params.contains("n_threads: 4,"), "{params}");
            assert!(params.contains("detect_language: false,"), "{params}");
            assert!(params.contains("prompt_n_tokens: 0,"), "{params}");
        }
    }

    #[test]
    fn the_glossary_prompt_reaches_every_window() {
        let params = format!("{:?}", engine::decode_params(None, false, 4, &[50, 51, 52]));
        assert!(params.contains("prompt_n_tokens: 3,"), "{params}");
        // The text of earlier windows still stays out.
        assert!(params.contains("no_context: true,"), "{params}");
    }

    #[test]
    fn quiet_input_releases_held_speech_without_waiting_for_more_audio() {
        let spool = Arc::new(Spool::new().unwrap());
        for i in 0..20 {
            spool.push(&speech(i * 100)).unwrap();
        }
        let (sent, windows) = std::sync::mpsc::channel();
        let reader = spool.clone();
        let worker = std::thread::spawn(move || {
            pump(
                &reader,
                &CancellationToken::new(),
                Duration::from_millis(100),
                |window| {
                    reader.complete(window.consumed);
                    sent.send((window.start_ms, window.consumed, window.speech))
                        .unwrap();
                    Ok(())
                },
            )
        });
        // Input is still open, as during Pause: only going idle can release these two seconds.
        let window = windows.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(window, (0, 20 * 1600, true));
        assert_eq!(spool.pending_ms(), 0);
        // Resuming after the pause starts a new window at its own time.
        spool.push(&speech(9_000)).unwrap();
        let window = windows.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(window, (9_000, 1600, true));
        spool.finish();
        worker.join().unwrap().unwrap();
        assert!(windows.try_recv().is_err());
    }

    #[test]
    fn a_backlog_is_worked_through_in_long_windows_then_normal_ones() {
        let spool = Spool::new().unwrap();
        for i in 0..400 {
            spool.push(&speech(i * 100)).unwrap();
        }
        spool.finish();
        let mut cuts = Vec::new();
        pump(
            &spool,
            &CancellationToken::new(),
            Duration::from_secs(5),
            |window| {
                spool.complete(window.consumed);
                cuts.push((window.start_ms, window.samples.len()));
                Ok(())
            },
        )
        .unwrap();
        // 40 s queued: one 28 s window, after which 13 s remain and 10 s windows resume.
        assert_eq!(
            cuts,
            [
                (0, 28 * 16_000),
                (27_000, 10 * 16_000),
                (36_000, 4 * 16_000)
            ]
        );
        assert_eq!(spool.pending_ms(), 0);
    }

    #[test]
    fn discarding_stops_the_pump_without_flushing() {
        let spool = Spool::new().unwrap();
        spool.push(&speech(0)).unwrap();
        spool.finish();
        let abort = CancellationToken::new();
        abort.cancel();
        let mut windows = 0;
        pump(&spool, &abort, Duration::from_millis(10), |_| {
            windows += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(windows, 0);
    }

    #[test]
    fn advertised_languages_match_the_native_multilingual_vocabulary() {
        let choices: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../../src/lib/whisperLanguages.json")).unwrap();
        let mut ids = std::collections::HashSet::new();
        for choice in choices {
            let code = choice["code"].as_str().unwrap();
            validate_language(Some(code)).unwrap();
            assert!(ids.insert(whisper_rs::get_lang_id(code).unwrap()));
        }
        assert_eq!(ids.len(), 99);
    }

    /// Put the pinned Tiny model in `dir`. Downloading it costs CI 40 to 100 s a lane, so when
    /// `WHISPER_SMOKE_MODEL_CACHE` names a directory, which CI restores between runs, a copy
    /// there that matches the pin is used instead, and a download refills it. The download
    /// itself stays covered, against a local server, by the tests in `models.rs`.
    async fn smoke_model(manager: &ModelManager, dir: &std::path::Path) {
        let id = ModelId::Tiny;
        let Some(cache) =
            std::env::var_os("WHISPER_SMOKE_MODEL_CACHE").filter(|path| !path.is_empty())
        else {
            manager.download(dir, id).await.unwrap();
            return;
        };
        let cache = std::path::PathBuf::from(cache);
        let cached = cache.join(id.file());
        let started = std::time::Instant::now();
        match models::read_verified(&cached, id) {
            Ok(bytes) => {
                // The bytes just verified, rather than a second read of the file.
                std::fs::write(dir.join(id.file()), bytes).unwrap();
                eprintln!("{} ({:.2?})", cached.display(), started.elapsed());
                return;
            }
            Err(error) => eprintln!("{}: {error:#}; downloading", cached.display()),
        }
        manager.download(dir, id).await.unwrap();
        eprintln!("{} downloaded ({:.2?})", id.file(), started.elapsed());
        std::fs::create_dir_all(&cache).expect("WHISPER_SMOKE_MODEL_CACHE must be writable");
        std::fs::copy(dir.join(id.file()), &cached)
            .expect("WHISPER_SMOKE_MODEL_CACHE must be writable");
    }

    /// The bundled speech-detection model, from the source tree.
    fn bundled_vad() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(vad::RESOURCE)
    }

    /// Explicit opt-in: downloads the 31 MiB pinned model (or takes it from CI's cache), then
    /// exercises the production spool, segmentation, speech detection, inference and
    /// EOF-draining path against both bundled fixtures, and translates the French one into
    /// English with its language detected. CI runs this on native x64 and ARM64 on the
    /// processor, and on Apple silicon with Metal, without a microphone or provider key.
    #[tokio::test]
    #[ignore = "downloads a Whisper model; run with --ignored local_whisper_smoke"]
    async fn local_whisper_smoke() {
        let placement = Placement {
            metal: cfg!(target_os = "macos"),
            helper: None,
            allow_software: false,
        };
        let expected = if cfg!(target_os = "macos") {
            "metal"
        } else {
            "cpu"
        };
        smoke(&placement, expected).await;
    }

    /// The same, through the Vulkan helper named by `WHISPER_VULKAN_HELPER` (built with
    /// `cargo build -p whisper-vulkan`). A software renderer is allowed, so CI's Linux lane runs
    /// it on Mesa's lavapipe with `GGML_VK_VISIBLE_DEVICES=0`: slow, but the whole path.
    #[tokio::test]
    #[ignore = "needs the Vulkan helper; run with --ignored local_whisper_vulkan_smoke"]
    async fn local_whisper_vulkan_smoke() {
        let helper = PathBuf::from(
            std::env::var_os("WHISPER_VULKAN_HELPER").expect("set WHISPER_VULKAN_HELPER"),
        );
        let placement = Placement {
            metal: false,
            helper: Some(&helper),
            allow_software: true,
        };
        smoke(&placement, "vulkan: ").await;
    }

    async fn smoke(placement: &Placement<'_>, expected_device: &str) {
        let temp = tempfile::tempdir().unwrap();
        // Non-ASCII on purpose: a Windows username can put such characters in the real path.
        let dir = temp.path().join("réunion-日本語");
        std::fs::create_dir(&dir).unwrap();
        let manager = ModelManager::default();
        smoke_model(&manager, &dir).await;
        let model = load_lease(
            manager.lease(&dir, ModelId::Tiny).unwrap(),
            Some(&bundled_vad()),
            placement,
        )
        .unwrap();
        assert!(model.vad.is_some(), "speech detection did not load");
        let device = model.device();
        eprintln!("device: {device}");
        assert!(device.starts_with(expected_device), "{device}");
        // The names-and-terms prompt, tokenized by the real model: the top of a long list fits.
        let long: Vec<String> = (0..100).map(|i| format!("Université numéro {i}")).collect();
        let prompt = glossary_prompt(&model, &long);
        assert!(
            prompt.len() > 10 && prompt.len() < long.len(),
            "{}",
            prompt.len()
        );
        assert!(prompt.tokens.len() <= 223);
        let glossary = ["IWAC".to_string(), "Voxtral".to_string()];
        for (fixture, language, translate, expected, terms) in [
            ("rehearsal-en.wav", None, false, Some("caption"), &[][..]),
            // The same recording with a prompt still reads as what was said.
            (
                "rehearsal-en.wav",
                None,
                false,
                Some("caption"),
                &glossary[..],
            ),
            ("rehearsal-fr.wav", Some("fr"), false, Some("public"), &[]),
            // Tiny translates poorly, and its wording moves with the instruction set: ARM64 said
            // "This is a recording of repetition…", x64 "This is an arrangement of repetition…".
            // So the translation is held to being English, not to any one word.
            ("rehearsal-fr.wav", None, true, None, &[]),
        ] {
            let samples = crate::audio::fixture::load_fixture(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("resources/fixtures")
                    .join(fixture),
            )
            .unwrap();
            let spool = Spool::new().unwrap();
            for (i, chunk) in samples.chunks(1600).enumerate() {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "the fixture is 16-bit PCM over 32768, so this restores it exactly"
                )]
                let pcm = chunk
                    .iter()
                    .flat_map(|s| ((*s * 32768.0) as i16).to_le_bytes())
                    .collect();
                spool
                    .push(&TimedChunk {
                        start_ms: i as u64 * 100,
                        pcm,
                    })
                    .unwrap();
            }
            spool.finish();
            let mut text = String::new();
            let mut last_end = 0;
            let started = std::time::Instant::now();
            transcribe(
                &model,
                &Task {
                    language,
                    translate,
                    glossary: terms,
                },
                &spool,
                &CancellationToken::new(),
                |caption, start, end, _| {
                    assert!(start >= last_end && end >= start);
                    assert!(end <= samples.len() as u64 / 16);
                    last_end = end;
                    text.push_str(caption);
                    text.push(' ');
                },
                || {},
            )
            .unwrap();
            // The time is the one number CI can compare across instruction-set changes.
            eprintln!("{fixture} ({:.2?}): {text}", started.elapsed());
            match expected {
                Some(word) => assert!(text.to_lowercase().contains(word), "{fixture}: {text}"),
                None => assert!(reads_as_english(&text), "{fixture}: {text}"),
            }
            assert_eq!(
                spool.pending_ms(),
                0,
                "Stop must drain the final partial window"
            );
        }
    }

    /// Whether `text` is English rather than French, judged by function words, which survive
    /// any rewording: plenty of English ones and few French ones.
    fn reads_as_english(text: &str) -> bool {
        const ENGLISH: [&str; 7] = ["the", "is", "and", "to", "of", "this", "a"];
        const FRENCH: [&str; 9] = ["le", "la", "les", "est", "et", "une", "des", "sont", "du"];
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
            .collect();
        let count = |set: &[&str]| words.iter().filter(|w| set.contains(w)).count();
        let (english, french) = (count(&ENGLISH), count(&FRENCH));
        english >= 5 && english > 3 * french
    }

    #[test]
    fn english_is_told_from_french_by_its_function_words() {
        // What Tiny produced on the two CI architectures, and the French it was translating.
        assert!(reads_as_english(concat!(
            "This is a recording of repetition for the subject system to direct. ",
            "A portrait is a public and each sentence is translated later. to the screen."
        )));
        assert!(reads_as_english(concat!(
            "This is an arrangement of repetition for the subject system to direct. ",
            "A portrait is a public and each sentence is translated later. ",
            "to the screen, then to the screen."
        )));
        assert!(!reads_as_english(concat!(
            "Ceci est un enregistrement de répétition pour le système de sous-titrage en direct. ",
            "Une oratrice s'adresse à un public et chaque phrase est transcrite puis affichée à ",
            "l'écran. Les sous-titres apparaissent juste après les mots."
        )));
    }

    #[test]
    fn validates_multilingual_choices_and_rejects_injection_or_unavailable_tokens() {
        for language in [
            None,
            Some("fr"),
            Some("en"),
            Some("ar"),
            Some("de"),
            Some("sw"),
            Some("ja"),
        ] {
            validate_language(language).unwrap();
        }
        for language in ["fake", "yue", "fr\0suffix"] {
            assert!(validate_language(Some(language)).is_err());
        }
    }
}
