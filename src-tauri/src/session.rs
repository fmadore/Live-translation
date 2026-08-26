//! Session orchestration: one bounded audio pipeline and realtime client per source.

use std::sync::{Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::future::join_all;
use tauri::async_runtime::JoinHandle as AsyncJoinHandle;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc::channel, Mutex as AsyncMutex};
use tokio_util::sync::CancellationToken;

use crate::audio::capture::run_microphone;
use crate::audio::fixture::run_rehearsal;
use crate::audio::loopback::run_system_loopback;
use crate::audio::AudioChunk;
use crate::gemini::{GeminiConfig, DEFAULT_HOST, DEFAULT_TRANSLATE_MODEL};
use crate::mistral::{
    MistralConfig, DEFAULT_MISTRAL_HOST, DEFAULT_MISTRAL_MODEL, DEFAULT_TARGET_STREAMING_DELAY_MS,
};
use crate::ondevice::{self, OnDeviceConfig};
use crate::openai::{
    OpenAiConfig, DEFAULT_OPENAI_HOST, DEFAULT_OPENAI_TRANSCRIBE_MODEL,
    DEFAULT_OPENAI_TRANSLATE_MODEL,
};
use crate::realtime::run_session;
use crate::secrets;
use crate::types::{
    events, AudioLevel, AudioSource, Origin, OutputMode, Provider, SessionState, StartOptions,
    StatusUpdate,
};

/// At most half a second of 100 ms chunks. The realtime consumer coalesces queued chunks
/// to the newest one after a stall, favoring live latency over replaying stale speech.
const AUDIO_CHANNEL_CAPACITY: usize = 5;
const LEVEL_CHANNEL_CAPACITY: usize = 8;
const CLIENT_DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct SessionManager {
    /// Serializes concurrent start/stop commands so an older request cannot tear down a
    /// newly-started session.
    lifecycle: AsyncMutex<()>,
    active: Mutex<Option<ActiveSession>>,
}

struct ActiveSession {
    cancel: CancellationToken,
    capture_threads: Vec<JoinHandle<()>>,
    /// Timer-driven audio producers used by commercial-provider rehearsal playback.
    fixture_tasks: Vec<AsyncJoinHandle<()>>,
    client_tasks: Vec<AsyncJoinHandle<()>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Report a failure on the audio side of one source: log it, tell the operator which source
/// died, and stop that source's client. The other source, if any, stays live.
///
/// The microphone gets its own wording on purpose. Under package identity Windows gates the
/// microphone per app, so a blocked install fails at device open with an ordinary cpal error
/// and nothing pointing at the toggle — see gate 6 in `docs/microsoft-store.md`. The message
/// covers a denied device and an absent one alike, because cpal reports both the same way,
/// and it keeps the underlying error so the real cause is still visible.
fn report_source_failure(
    app: &AppHandle,
    origin: Origin,
    error: &anyhow::Error,
    cancel: &CancellationToken,
) {
    tracing::error!(?origin, "capture failed: {error:#}");
    let message = match origin {
        Origin::Microphone => format!(
            "Microphone capture failed ({error:#}). If access is blocked, enable it under \
             Windows Settings > Privacy & security > Microphone \
             (ms-settings:privacy-microphone), then start again."
        ),
        Origin::System => format!("{origin:?} capture: {error}"),
    };
    let _ = app.emit(
        events::STATUS,
        StatusUpdate {
            state: SessionState::Error,
            message: Some(message),
            origin: Some(origin),
        },
    );
    cancel.cancel();
}

impl SessionManager {
    pub async fn start(&self, app: &AppHandle, options: StartOptions) -> Result<()> {
        let _lifecycle = self.lifecycle.lock().await;
        self.stop_active(app).await;

        match (options.mode, options.provider) {
            (OutputMode::Translate, Provider::Mistral) => {
                anyhow::bail!("Mistral Voxtral is transcription-only; choose Live subtitles")
            }
            (OutputMode::Translate, Provider::OnDevice) => {
                anyhow::bail!(
                    "The built-in demonstration is same-language only; choose Live subtitles, \
                     or Gemini/OpenAI to translate"
                )
            }
            // Guarded on the capability rather than a provider list, so adding another
            // translating backend cannot silently become a valid subtitle engine.
            (OutputMode::Transcribe, provider) if provider.can_translate() => {
                anyhow::bail!("Live subtitles use Mistral or the built-in demonstration")
            }
            _ => {}
        }
        if options.provider == Provider::OnDevice && options.source != AudioSource::Microphone {
            anyhow::bail!("The built-in demonstration uses its bundled sample; select Demo audio")
        }
        if options.provider == Provider::OnDevice && options.rehearsal.is_some() {
            anyhow::bail!("The built-in demonstration already uses bundled content")
        }

        let provider = options.provider;
        // The built-in demonstration is the one backend that starts with no credential.
        let api_key = if provider.requires_api_key() {
            secrets::resolve_api_key(provider)?
        } else {
            String::new()
        };
        let target_rate = provider.input_sample_rate();
        let target_code = options.target_language.bcp47().to_string();

        let gemini_host =
            std::env::var("GEMINI_WS_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
        let gemini_model = std::env::var("GEMINI_TRANSLATE_MODEL")
            .unwrap_or_else(|_| DEFAULT_TRANSLATE_MODEL.to_string());
        let openai_host =
            std::env::var("OPENAI_WS_HOST").unwrap_or_else(|_| DEFAULT_OPENAI_HOST.to_string());
        let openai_model = std::env::var("OPENAI_TRANSLATE_MODEL")
            .unwrap_or_else(|_| DEFAULT_OPENAI_TRANSLATE_MODEL.to_string());
        let openai_transcribe = std::env::var("OPENAI_TRANSCRIBE_MODEL")
            .unwrap_or_else(|_| DEFAULT_OPENAI_TRANSCRIBE_MODEL.to_string());
        let mistral_host =
            std::env::var("MISTRAL_WS_HOST").unwrap_or_else(|_| DEFAULT_MISTRAL_HOST.to_string());
        let mistral_model = std::env::var("MISTRAL_TRANSCRIBE_MODEL")
            .unwrap_or_else(|_| DEFAULT_MISTRAL_MODEL.to_string());
        let mistral_delay = std::env::var("MISTRAL_TARGET_STREAMING_DELAY_MS")
            .ok()
            .map(|value| value.parse::<u32>())
            .transpose()
            .context("MISTRAL_TARGET_STREAMING_DELAY_MS must be an integer")?
            .unwrap_or(DEFAULT_TARGET_STREAMING_DELAY_MS);

        let cancel = CancellationToken::new();
        let cancel_guard = cancel.clone().drop_guard();
        let mut capture_threads = Vec::new();
        let mut fixture_tasks = Vec::new();
        let mut client_tasks = Vec::new();

        let (level_tx, mut level_rx) = channel::<AudioLevel>(LEVEL_CHANNEL_CAPACITY);
        let level_app = app.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(level) = level_rx.recv().await {
                let _ = level_app.emit(events::LEVEL, &level);
            }
        });

        let mut spawn_source = |origin: Origin| -> Result<()> {
            let (audio_tx, audio_rx) = channel::<AudioChunk>(AUDIO_CHANNEL_CAPACITY);
            let source_cancel = cancel.child_token();

            if provider == Provider::OnDevice {
                // The deterministic demo emits its own level/caption timeline and never
                // opens a capture device. Close the unused producer immediately.
                drop(audio_tx);
            } else {
                match options.rehearsal {
                    // Rehearsal swaps the capture device for a bundled recording and changes
                    // nothing else: same channel, same chunk shape, same engine below it.
                    Some(language) => {
                        let fixture_app = app.clone();
                        let fixture_cancel = source_cancel.clone();
                        let fixture_level_tx = level_tx.clone();
                        fixture_tasks.push(tauri::async_runtime::spawn(async move {
                            let result = run_rehearsal(
                                &fixture_app,
                                language,
                                target_rate,
                                fixture_level_tx,
                                audio_tx,
                                fixture_cancel.clone(),
                            )
                            .await;
                            if let Err(error) = result {
                                report_source_failure(
                                    &fixture_app,
                                    origin,
                                    &error,
                                    &fixture_cancel,
                                );
                            }
                        }));
                    }
                    None => {
                        let capture_app = app.clone();
                        let capture_cancel = source_cancel.clone();
                        let capture_error_cancel = source_cancel.clone();
                        let capture_level_tx = level_tx.clone();
                        let mic_name = options.mic_device_name.clone();
                        let handle = std::thread::Builder::new()
                            .name(format!("capture-{origin:?}"))
                            .spawn(move || {
                                let result = match origin {
                                    Origin::Microphone => run_microphone(
                                        capture_app.clone(),
                                        mic_name,
                                        target_rate,
                                        capture_level_tx,
                                        audio_tx,
                                        capture_cancel,
                                    ),
                                    Origin::System => run_system_loopback(
                                        target_rate,
                                        capture_level_tx,
                                        audio_tx,
                                        capture_cancel,
                                    ),
                                };
                                if let Err(error) = result {
                                    report_source_failure(
                                        &capture_app,
                                        origin,
                                        &error,
                                        &capture_error_cancel,
                                    );
                                }
                            })
                            .context("failed to spawn capture thread")?;
                        capture_threads.push(handle);
                    }
                }
            }

            let client_app = app.clone();
            match provider {
                Provider::Gemini => {
                    let config = GeminiConfig {
                        api_key: api_key.clone(),
                        model: gemini_model.clone(),
                        host: gemini_host.clone(),
                        target_language_code: target_code.clone(),
                        origin,
                    };
                    client_tasks.push(tauri::async_runtime::spawn(run_session(
                        client_app,
                        config,
                        audio_rx,
                        source_cancel,
                    )));
                }
                Provider::OpenAi => {
                    let config = OpenAiConfig {
                        api_key: api_key.clone(),
                        model: openai_model.clone(),
                        transcribe_model: openai_transcribe.clone(),
                        host: openai_host.clone(),
                        target_language_code: target_code.clone(),
                        origin,
                    };
                    client_tasks.push(tauri::async_runtime::spawn(run_session(
                        client_app,
                        config,
                        audio_rx,
                        source_cancel,
                    )));
                }
                Provider::Mistral => {
                    let config = MistralConfig {
                        api_key: api_key.clone(),
                        model: mistral_model.clone(),
                        host: mistral_host.clone(),
                        target_streaming_delay_ms: mistral_delay,
                        origin,
                        received_delta: false,
                    };
                    client_tasks.push(tauri::async_runtime::spawn(run_session(
                        client_app,
                        config,
                        audio_rx,
                        source_cancel,
                    )));
                }
                Provider::OnDevice => {
                    let config = OnDeviceConfig {
                        origin,
                        language: options.target_language,
                    };
                    client_tasks.push(tauri::async_runtime::spawn(ondevice::run_session(
                        client_app,
                        config,
                        audio_rx,
                        source_cancel,
                    )));
                }
            }
            Ok(())
        };

        // A rehearsal runs exactly one origin, System, off the bundled fixture: `source` and
        // the microphone selection are deliberately ignored, because the point of the mode is
        // to exercise the pipeline with no audio hardware involved at all.
        if options.rehearsal.is_some() {
            spawn_source(Origin::System)?;
        } else {
            if options.source.wants_mic() {
                spawn_source(Origin::Microphone)?;
            }
            if options.source.wants_system() {
                spawn_source(Origin::System)?;
            }
        }

        let cancel = cancel_guard.disarm();
        *lock(&self.active) = Some(ActiveSession {
            cancel,
            capture_threads,
            fixture_tasks,
            client_tasks,
        });
        Ok(())
    }

    pub async fn stop(&self, app: &AppHandle) {
        let _lifecycle = self.lifecycle.lock().await;
        self.stop_active(app).await;
    }

    async fn stop_active(&self, app: &AppHandle) {
        let session = lock(&self.active).take();
        if let Some(mut session) = session {
            session.cancel.cancel();
            let capture_threads = session.capture_threads;
            if let Err(error) = tauri::async_runtime::spawn_blocking(move || {
                for handle in capture_threads {
                    if handle.join().is_err() {
                        tracing::warn!("capture thread panicked while stopping");
                    }
                }
            })
            .await
            {
                tracing::warn!("capture join task failed: {error}");
            }

            // Rehearsal playback holds the producer end of its audio channel, and the client
            // below only sees the stream end once that is dropped — so drain it here, in the
            // same place the capture threads are joined.
            for result in join_all(session.fixture_tasks.iter_mut()).await {
                if let Err(error) = result {
                    tracing::warn!("rehearsal playback task failed: {error}");
                }
            }

            // Providers may emit their last transcript while flushing. Do not report Idle
            // (or start a replacement session) until that bounded drain has completed.
            if tokio::time::timeout(
                CLIENT_DRAIN_TIMEOUT,
                join_all(session.client_tasks.iter_mut()),
            )
            .await
            .is_err()
            {
                tracing::warn!("realtime clients did not finish graceful shutdown in time");
                for task in &session.client_tasks {
                    task.abort();
                }
            }
            let _ = app.emit(
                events::STATUS,
                StatusUpdate {
                    state: SessionState::Idle,
                    message: None,
                    origin: None,
                },
            );
            tracing::info!("session stopped");
        }
    }
}
