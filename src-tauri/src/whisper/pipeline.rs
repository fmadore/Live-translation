//! A local source's pipeline: capture written to the spool, the spool cut into windows, each
//! window decoded and reconciled into captions, and how the source ended reported.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use super::backend::LoadedModel;
use super::engine;
use super::language::Languages;
use super::prompt::Prompt;
use super::reconcile::Reconciler;
use super::segment::{Segmenter, Window};
use super::spool::{Next, Spool, TimedChunk};
use super::validate_language;
use crate::errors::{AppError, id};
use crate::realtime::Events;
use crate::sync::lock;
use crate::types::{Caption, Origin, SessionState, StatusUpdate, events};

/// Input quiet for this long releases whatever audio is held, so the last words before Pause,
/// or before a call goes silent, are captioned now rather than when audio resumes.
const IDLE: Duration = Duration::from_secs(1);

/// A backlog beyond this switches to long windows; see `segment::LONG_WINDOW`.
const LONG_WINDOW_BACKLOG_MS: u64 = 30_000;

/// ggml's workers busy-wait between operations, so a thread per hardware thread starves
/// capture and the interface, and beyond about eight these small models stop getting faster.
/// x64 processors are assumed to have SMT: half the logical processors is one thread per core,
/// and the sibling hardware threads absorb capture and the interface. Windows-on-ARM has no
/// SMT, so one core is left free instead.
pub(super) fn inference_threads(logical: usize) -> i32 {
    let threads = if cfg!(target_arch = "x86_64") {
        logical / 2
    } else {
        logical.saturating_sub(1)
    };
    i32::try_from(threads).unwrap_or(i32::MAX).clamp(1, 8)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Progress {
    pub(super) origin: Origin,
    pub(super) pending_ms: u64,
    pub(super) finalizing: bool,
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
pub(super) trait LocalEvents: Events + Clone + Send + 'static {
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
    let _ = app.emit(events::WHISPER_ENGINE, model.engine());
    let engine_app = app.clone();
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
            &mut || {
                report();
                // The graphics processor failed on a window: say so once, whichever source
                // decoded it. The other source's report finds the flag already cleared.
                if model.engine_changed.swap(false, Ordering::Relaxed) {
                    let _ = engine_app.emit(events::WHISPER_ENGINE, model.engine());
                }
            },
        )
    })
    .await;
}

/// A local source without its model: everything `drive` needs besides the inference itself.
pub(super) struct Pipeline<E> {
    pub(super) events: E,
    pub(super) origin: Origin,
    pub(super) input: Receiver<TimedChunk>,
    pub(super) spool: Arc<Spool>,
    pub(super) capture_cancel: CancellationToken,
    pub(super) abort: CancellationToken,
    pub(super) failure: Arc<Mutex<Option<AppError>>>,
}

/// One caption from the inference: its text, start and end in session milliseconds, and turn.
pub(super) type Emit<'a> = &'a mut dyn FnMut(&str, u64, u64, u64);
/// Called after each window, to report the backlog.
pub(super) type Report<'a> = &'a mut dyn FnMut();

/// Write the input to the spool on one blocking worker, run `infer` over it on another, and
/// report how the source ended once both have: Idle after a full drain, or Error with why —
/// a failed write, a capture failure recorded on the spool, a failed or panicked inference, or
/// the operator discarding what was left. `infer` turns the spool into captions as `transcribe`
/// does, reading it until it ends or the abort token fires; the tests stand in for Whisper
/// with it.
pub(super) async fn drive<E: LocalEvents>(
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
pub(super) struct Task<'a> {
    pub(super) language: Option<&'a str>,
    pub(super) translate: bool,
    pub(super) glossary: &'a [String],
}

pub(super) fn transcribe(
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
pub(super) fn glossary_prompt(model: &LoadedModel, glossary: &[String]) -> Prompt {
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
pub(super) fn pump(
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
    if !abort.is_cancelled()
        && let Some(window) = segmenter.finish()
    {
        process(window)?;
    }
    Ok(())
}
