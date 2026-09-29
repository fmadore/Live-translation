//! Multilingual, CPU-only local transcription. Capture, disk ingestion and inference run
//! independently; normal Stop closes input then drains every accepted frame.
pub mod models;
mod segment;
pub mod spool;

use std::sync::{atomic::Ordering, Arc, Mutex};

use anyhow::{ensure, Context, Result};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::errors::{id, AppError};
use crate::types::{events, Caption, Origin, SessionState, StatusUpdate};
use models::{ModelId, ModelLease, ModelManager};
use segment::{trim_overlap, Segmenter};
use spool::{Spool, TimedChunk};

pub struct LoadedModel {
    context: WhisperContext,
    _lease: ModelLease,
}

pub fn load(app: &AppHandle, id: ModelId) -> Result<Arc<LoadedModel>> {
    let lease = app
        .state::<ModelManager>()
        .lease(&models::directory(app)?, id)?;
    load_lease(lease)
}

fn load_lease(lease: ModelLease) -> Result<Arc<LoadedModel>> {
    // Read through Rust's Unicode-aware filesystem API (Windows usernames may contain
    // non-ASCII characters), then pass the exact verified bytes to the native loader.
    let bytes = lease.read_verified()?;
    let mut params = WhisperContextParameters::default();
    params.use_gpu(false);
    let context = WhisperContext::new_from_buffer_with_params(&bytes, params)
        .context("Could not load Whisper model")?;
    Ok(Arc::new(LoadedModel {
        context,
        _lease: lease,
    }))
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

fn progress(app: &AppHandle, origin: Origin, spool: &Spool) {
    let _ = app.emit(
        events::WHISPER_PROGRESS,
        Progress {
            origin,
            pending_ms: spool.pending_ms(),
            finalizing: spool.finalizing.load(Ordering::Relaxed),
        },
    );
}

fn status(app: &AppHandle, origin: Origin, state: SessionState, message: Option<AppError>) {
    let _ = app.emit(
        events::STATUS,
        StatusUpdate {
            state,
            message,
            origin: Some(origin),
            lane: None,
        },
    );
}

pub struct LocalSession {
    pub app: AppHandle,
    pub model: Arc<LoadedModel>,
    pub language: Option<String>,
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
        origin,
        mut input,
        spool,
        capture_cancel,
        abort,
        failure,
    } = config;
    status(&app, origin, SessionState::Running, None);
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
                ingest_spool.push(frame)?;
                count += 1;
                if count.is_multiple_of(10) {
                    progress(&ingest_app, origin, &ingest_spool);
                }
            }
            ensure!(
                !ingest_spool.overflowed.load(Ordering::Relaxed),
                "Audio capture outran the local disk buffer; the transcript may be incomplete"
            );
            if let Some(error) = ingest_spool
                .capture_error
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref()
            {
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
        transcribe(
            &model,
            language.as_deref(),
            &worker_spool,
            &worker_abort,
            |text, start, end, turn| {
                let _ = worker_app.emit(
                    events::CAPTION,
                    Caption {
                        turn_id: turn,
                        text,
                        source_text: "",
                        final_: true,
                        origin,
                        lane: 0,
                        start_ms: start,
                        end_ms: end,
                    },
                );
            },
            || progress(&worker_app, origin, &worker_spool),
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
        *failure.lock().unwrap_or_else(|e| e.into_inner()) =
            Some(AppError::with(id::WHISPER_SESSION, format!("{error:#}")));
        status(
            &app,
            origin,
            SessionState::Error,
            Some(AppError::with(id::WHISPER_SESSION, format!("{error:#}"))),
        );
    } else {
        status(&app, origin, SessionState::Idle, None);
    }
}

fn transcribe(
    model: &LoadedModel,
    language: Option<&str>,
    spool: &Spool,
    abort: &CancellationToken,
    mut emit: impl FnMut(&str, u64, u64, u64),
    mut report: impl FnMut(),
) -> Result<()> {
    let mut state = model.context.create_state()?;
    let mut segmenter = Segmenter::default();
    let mut turn = 0u64;
    let mut previous_text = String::new();
    let mut previous_end = 0u64;
    let threads = std::thread::available_parallelism()
        .map_or(2, usize::from)
        .clamp(1, 4) as i32;
    let mut process = |window: segment::Window| -> Result<()> {
        if abort.is_cancelled() {
            return Ok(());
        }
        if window.speech {
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_language(language);
            // `detect_language=true` asks whisper.cpp to return after detection. Leaving it
            // false with language=None performs detection followed by transcription.
            params.set_translate(false);
            params.set_no_context(true);
            params.set_n_threads(threads);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);
            params.set_print_special(false);
            params.set_suppress_blank(true);
            params.set_suppress_nst(true);
            // whisper-rs 0.16's closure helper has an incorrect user-data cast and leaks its
            // allocation. Use the C callback with a borrowed, thread-safe token instead.
            // SAFETY: full() is synchronous; abort outlives the entire call, and the callback
            // only reads CancellationToken. No pointer escapes this invocation.
            unsafe {
                params.set_abort_callback(Some(abort_requested));
                params.set_abort_callback_user_data(std::ptr::from_ref(abort).cast_mut().cast());
            }
            let mut samples = window.samples;
            // Whisper needs enough samples for its analysis window, including a final
            // phrase shorter than a second. Padding affects inference only, not timestamps.
            let duration_ms = samples.len() as u64 / 16;
            samples.resize(samples.len().max(16_000), 0.0);
            if let Err(error) = state.full(params, &samples) {
                if abort.is_cancelled() {
                    return Ok(());
                }
                return Err(error.into());
            }
            for segment in state.as_iter() {
                if segment.no_speech_probability() > 0.8 {
                    continue;
                }
                let start = window.start_ms
                    + (segment.start_timestamp().max(0) as u64 * 10).min(duration_ms);
                let end =
                    window.start_ms + (segment.end_timestamp().max(0) as u64 * 10).min(duration_ms);
                if end <= previous_end {
                    continue;
                }
                let text = segment.to_str()?.trim();
                let text = trim_overlap(&previous_text, text, start < previous_end);
                if text.is_empty() {
                    continue;
                }
                turn += 1;
                emit(
                    text,
                    start.max(previous_end),
                    end.max(start.max(previous_end)),
                    turn,
                );
                previous_text = text.to_owned();
                previous_end = end;
            }
        }
        spool.complete(window.consumed);
        report();
        Ok(())
    };
    while !abort.is_cancelled() {
        let Some(frame) = spool.next()? else {
            break;
        };
        for window in segmenter.push(frame) {
            process(window)?;
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

    /// Explicit opt-in: downloads the 31 MiB pinned model, then exercises the production
    /// spool, segmentation, inference and EOF-draining path against both bundled fixtures.
    /// CI runs this on native x64 and ARM64, without a GPU, microphone or provider key.
    #[tokio::test]
    #[ignore = "downloads a Whisper model; run with --ignored local_whisper_smoke"]
    async fn local_whisper_smoke() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("réunion-日本語");
        std::fs::create_dir(&dir).unwrap();
        let manager = ModelManager::default();
        manager.download(&dir, ModelId::Tiny).await.unwrap();
        let model = load_lease(manager.lease(&dir, ModelId::Tiny).unwrap()).unwrap();
        for (fixture, language, expected) in [
            ("rehearsal-en.wav", None, "caption"),
            ("rehearsal-fr.wav", Some("fr"), "public"),
        ] {
            let samples = crate::audio::fixture::load_fixture(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("resources/fixtures")
                    .join(fixture),
            )
            .unwrap();
            let spool = Spool::new().unwrap();
            for (i, chunk) in samples.chunks(1600).enumerate() {
                spool
                    .push(TimedChunk {
                        start_ms: i as u64 * 100,
                        pcm: chunk
                            .iter()
                            .flat_map(|s| ((*s * 32768.0) as i16).to_le_bytes())
                            .collect(),
                    })
                    .unwrap();
            }
            spool.finish();
            let mut text = String::new();
            let mut last_end = 0;
            transcribe(
                &model,
                language,
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
            eprintln!("{fixture}: {text}");
            assert!(text.to_lowercase().contains(expected), "{fixture}: {text}");
            assert_eq!(
                spool.pending_ms(),
                0,
                "Stop must drain the final partial window"
            );
        }
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
