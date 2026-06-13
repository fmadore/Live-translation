//! Session orchestration: wires audio capture (one thread per active source) to a Gemini
//! Live client (one task per active source). A single `CancellationToken` tears everything
//! down cleanly on stop.

use std::sync::Mutex;
use std::thread::JoinHandle;

use anyhow::{Context, Result};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;

use crate::audio::capture::run_microphone;
use crate::audio::loopback::run_system_loopback;
use crate::audio::AudioChunk;
use crate::gemini::{run_session, GeminiConfig, DEFAULT_HOST, DEFAULT_MODEL};
use crate::secrets;
use crate::types::{events, Origin, SessionState, StartOptions, StatusUpdate};

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

        let api_key = secrets::resolve_api_key()?;
        let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        let host = std::env::var("GEMINI_WS_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
        let target = options.target_language.bcp47().to_string();

        let cancel = CancellationToken::new();
        let mut capture_threads = Vec::new();

        let mut spawn_source = |origin: Origin| -> Result<()> {
            let (tx, rx) = unbounded_channel::<AudioChunk>();

            // Capture thread (blocking; owns the cpal/WASAPI stream).
            let cap_app = app.clone();
            let cap_cancel = cancel.clone();
            let mic_name = options.mic_device_name.clone();
            let handle = std::thread::Builder::new()
                .name(format!("capture-{origin:?}"))
                .spawn(move || {
                    let result = match origin {
                        Origin::Microphone => {
                            run_microphone(cap_app.clone(), mic_name, tx, cap_cancel)
                        }
                        Origin::System => run_system_loopback(cap_app.clone(), tx, cap_cancel),
                    };
                    if let Err(e) = result {
                        tracing::error!(?origin, "capture failed: {e:#}");
                        let _ = cap_app.emit(
                            events::STATUS,
                            StatusUpdate {
                                state: SessionState::Error,
                                message: Some(format!("{origin:?} capture: {e}")),
                            },
                        );
                    }
                })
                .context("failed to spawn capture thread")?;
            capture_threads.push(handle);

            // Gemini client task for this source.
            let cfg = GeminiConfig {
                api_key: api_key.clone(),
                model: model.clone(),
                host: host.clone(),
                target_language_code: target.clone(),
                origin,
            };
            let client_app = app.clone();
            let client_cancel = cancel.clone();
            tauri::async_runtime::spawn(run_session(client_app, cfg, rx, client_cancel));
            Ok(())
        };

        if options.source.wants_mic() {
            spawn_source(Origin::Microphone)?;
        }
        if options.source.wants_system() {
            spawn_source(Origin::System)?;
        }

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
                },
            );
            tracing::info!("session stopped");
        }
    }
}
