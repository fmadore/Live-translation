//! Gemini Live WebSocket client: opens a bidirectional stream, sends 16 kHz PCM chunks,
//! and turns the returned transcriptions into caption events. Auto-reconnects on drop.

use std::time::Duration;

use anyhow::{Context, Result};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

use super::protocol::{RealtimeInputMessage, ServerMessage, SetupMessage};
use crate::audio::AudioChunk;
use crate::types::{events, Caption, Origin, SessionState, StatusUpdate};

pub const DEFAULT_MODEL: &str = "gemini-3.5-live-translate-preview";
pub const DEFAULT_HOST: &str = "generativelanguage.googleapis.com";

#[derive(Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    pub host: String,
    pub target_language_code: String,
    pub origin: Origin,
}

impl GeminiConfig {
    fn ws_url(&self) -> String {
        format!(
            "wss://{}/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key={}",
            self.host, self.api_key
        )
    }
}

/// Accumulates transcription deltas for the current turn.
#[derive(Default)]
struct TurnAccumulator {
    id: u64,
    source: String,
    translated: String,
}

/// Run a session for one audio source, reconnecting until cancelled.
pub async fn run_session(
    app: AppHandle,
    cfg: GeminiConfig,
    mut audio_rx: UnboundedReceiver<AudioChunk>,
    cancel: CancellationToken,
) {
    let mut backoff = Duration::from_secs(1);
    let mut first = true;

    while !cancel.is_cancelled() {
        emit_status(
            &app,
            if first {
                SessionState::Connecting
            } else {
                SessionState::Reconnecting
            },
            None,
        );

        match connect_and_run(&app, &cfg, &mut audio_rx, &cancel).await {
            Ok(()) => {
                // Clean close. If we weren't cancelled, the server hung up — reconnect.
                if cancel.is_cancelled() {
                    break;
                }
                tracing::warn!(origin = ?cfg.origin, "Gemini stream closed; reconnecting");
            }
            Err(e) => {
                tracing::error!(origin = ?cfg.origin, "Gemini stream error: {e:#}");
                emit_status(&app, SessionState::Reconnecting, Some(format!("{e}")));
            }
        }

        first = false;
        if cancel.is_cancelled() {
            break;
        }

        // Backoff, but stay responsive to cancellation.
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = tokio::time::sleep(backoff) => {}
        }
        backoff = (backoff * 2).min(Duration::from_secs(16));
    }

    tracing::info!(origin = ?cfg.origin, "session loop ended");
}

async fn connect_and_run(
    app: &AppHandle,
    cfg: &GeminiConfig,
    audio_rx: &mut UnboundedReceiver<AudioChunk>,
    cancel: &CancellationToken,
) -> Result<()> {
    let (ws, _resp) = connect_async(cfg.ws_url())
        .await
        .context("WebSocket connect failed")?;
    let (mut write, mut read) = ws.split();

    // Hand the model its translation configuration before any audio.
    let setup = SetupMessage::new(&cfg.model, &cfg.target_language_code);
    write
        .send(Message::Text(serde_json::to_string(&setup)?))
        .await
        .context("failed to send setup")?;

    tracing::info!(origin = ?cfg.origin, target = %cfg.target_language_code, "connected to Gemini Live");
    emit_status(app, SessionState::Running, None);

    let mut acc = TurnAccumulator::default();

    loop {
        tokio::select! {
            biased;

            _ = cancel.cancelled() => {
                let _ = write.send(Message::Close(None)).await;
                return Ok(());
            }

            maybe_chunk = audio_rx.recv() => {
                match maybe_chunk {
                    Some(chunk) => {
                        let data = base64::engine::general_purpose::STANDARD.encode(&chunk.pcm_le);
                        let msg = RealtimeInputMessage::pcm16(data);
                        write
                            .send(Message::Text(serde_json::to_string(&msg)?))
                            .await
                            .context("failed to send audio chunk")?;
                    }
                    // All capture senders dropped: the session is stopping.
                    None => {
                        let _ = write.send(Message::Close(None)).await;
                        return Ok(());
                    }
                }
            }

            maybe_msg = read.next() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_server_message(app, cfg.origin, &text, &mut acc);
                    }
                    Some(Ok(Message::Binary(bytes))) => {
                        // The server occasionally frames JSON as binary.
                        if let Ok(text) = String::from_utf8(bytes) {
                            handle_server_message(app, cfg.origin, &text, &mut acc);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => return Ok(()),
                    Some(Ok(_)) => { /* ping/pong handled by the library */ }
                    Some(Err(e)) => return Err(e).context("WebSocket read error"),
                }
            }
        }
    }
}

fn handle_server_message(app: &AppHandle, origin: Origin, text: &str, acc: &mut TurnAccumulator) {
    let msg: ServerMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            tracing::debug!("unparsed server message: {e} :: {text}");
            return;
        }
    };

    if msg.setup_complete.is_some() {
        tracing::debug!(?origin, "Gemini setup complete; streaming audio");
    }
    if msg.go_away.is_some() {
        tracing::info!("Gemini sent goAway; will reconnect");
    }

    let Some(content) = msg.server_content else {
        return;
    };

    if let Some(t) = &content.input_transcription {
        acc.source.push_str(&t.text);
    }
    if let Some(t) = &content.output_transcription {
        acc.translated.push_str(&t.text);
    }

    let turn_complete = content.turn_complete.unwrap_or(false);

    // Emit whenever we have new translated text, or to mark the turn final.
    if content.output_transcription.is_some()
        || content.input_transcription.is_some()
        || turn_complete
    {
        let caption = Caption {
            turn_id: acc.id,
            text: acc.translated.clone(),
            source_text: acc.source.clone(),
            final_: turn_complete,
            origin,
        };
        let _ = app.emit(events::CAPTION, caption.to_json());
    }

    if turn_complete {
        acc.id += 1;
        acc.source.clear();
        acc.translated.clear();
    }
}

fn emit_status(app: &AppHandle, state: SessionState, message: Option<String>) {
    let _ = app.emit(events::STATUS, StatusUpdate { state, message });
}
