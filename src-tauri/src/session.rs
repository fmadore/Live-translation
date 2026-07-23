//! Session orchestration: wires audio capture (one thread per active source) to a realtime
//! translation client (one task per active source). A single `CancellationToken` tears
//! everything down cleanly on stop.

use std::sync::Mutex;
use std::thread::JoinHandle;

use anyhow::{Context, Result};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;

use crate::audio::capture::run_microphone;
use crate::audio::loopback::run_system_loopback;
use crate::audio::AudioChunk;
use crate::gemini::{GeminiConfig, DEFAULT_HOST, DEFAULT_TRANSLATE_MODEL};
use crate::openai::{
    OpenAiConfig, DEFAULT_OPENAI_HOST, DEFAULT_OPENAI_TRANSCRIBE_MODEL,
    DEFAULT_OPENAI_TRANSLATE_MODEL,
};
use crate::realtime::run_session;
use crate::secrets;
use crate::types::{
    events, AudioLevel, Origin, Provider, SessionState, StartOptions, StatusUpdate,
};

#[derive(Default)]
pub struct SessionManager {
    active: Mutex<Option<ActiveSession>>,
}

struct ActiveSession {
    cancel: CancellationToken,
    capture_threads: Vec<JoinHandle<()>>,
}

impl SessionManager {
    /// Start translating the selected source(s). Idempotent: stops any prior session first.
    pub fn start(&self, app: &AppHandle, options: StartOptions) -> Result<()> {
        self.stop(app);

        let provider = options.provider;
        let api_key = secrets::resolve_api_key(provider)?;
        let target_rate = provider.input_sample_rate();
        let target_code = options.target_language.bcp47().to_string();

        // Provider-specific connection details; only the selected provider's are used.
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

        let cancel = CancellationToken::new();
        // If anything below fails we return early with sources already spawned; the guard
        // cancels them on the error path (dropping a token alone does NOT cancel it).
        let cancel_guard = cancel.clone().drop_guard();
        let mut capture_threads = Vec::new();

        // Level-meter events are forwarded here so the real-time capture callbacks never
        // touch webview IPC themselves. The task ends when the last sender drops.
        let (level_tx, mut level_rx) = unbounded_channel::<AudioLevel>();
        let level_app = app.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(level) = level_rx.recv().await {
                let _ = level_app.emit(events::LEVEL, &level);
            }
        });

        let mut spawn_source = |origin: Origin| -> Result<()> {
            let (tx, rx) = unbounded_channel::<AudioChunk>();

            // Capture thread (blocking; owns the cpal/WASAPI stream). Resamples to the
            // provider's input rate (16 kHz Gemini / 24 kHz OpenAI).
            let cap_app = app.clone();
            let cap_cancel = cancel.clone();
            let cap_level_tx = level_tx.clone();
            let mic_name = options.mic_device_name.clone();
            let handle = std::thread::Builder::new()
                .name(format!("capture-{origin:?}"))
                .spawn(move || {
                    let result = match origin {
                        Origin::Microphone => run_microphone(
                            cap_app.clone(),
                            mic_name,
                            target_rate,
                            cap_level_tx,
                            tx,
                            cap_cancel,
                        ),
                        Origin::System => {
                            run_system_loopback(target_rate, cap_level_tx, tx, cap_cancel)
                        }
                    };
                    if let Err(e) = result {
                        tracing::error!(?origin, "capture failed: {e:#}");
                        let _ = cap_app.emit(
                            events::STATUS,
                            StatusUpdate {
                                state: SessionState::Error,
                                message: Some(format!("{origin:?} capture: {e}")),
                                origin: Some(origin),
                            },
                        );
                    }
                })
                .context("failed to spawn capture thread")?;
            capture_threads.push(handle);

            // Translation client task for this source, dispatched by provider.
            let client_app = app.clone();
            let client_cancel = cancel.clone();
            match provider {
                Provider::Gemini => {
                    let cfg = GeminiConfig {
                        api_key: api_key.clone(),
                        model: gemini_model.clone(),
                        host: gemini_host.clone(),
                        target_language_code: target_code.clone(),
                        origin,
                    };
                    tauri::async_runtime::spawn(run_session(client_app, cfg, rx, client_cancel));
                }
                Provider::OpenAi => {
                    let cfg = OpenAiConfig {
                        api_key: api_key.clone(),
                        model: openai_model.clone(),
                        transcribe_model: openai_transcribe.clone(),
                        host: openai_host.clone(),
                        target_language_code: target_code.clone(),
                        origin,
                    };
                    tauri::async_runtime::spawn(run_session(client_app, cfg, rx, client_cancel));
                }
            }
            Ok(())
        };

        if options.source.wants_mic() {
            spawn_source(Origin::Microphone)?;
        }
        if options.source.wants_system() {
            spawn_source(Origin::System)?;
        }

        // Everything is up: hand cancellation ownership to the stored session.
        let cancel = cancel_guard.disarm();
        *self.active.lock().unwrap() = Some(ActiveSession {
            cancel,
            capture_threads,
        });
        Ok(())
    }

    /// Stop the current session (no-op if idle). Cancels everything and joins capture threads.
    pub fn stop(&self, app: &AppHandle) {
        let session = self.active.lock().unwrap().take();
        if let Some(session) = session {
            session.cancel.cancel();
            for handle in session.capture_threads {
                let _ = handle.join();
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
