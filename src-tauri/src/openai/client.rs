//! OpenAI Realtime *translations* client: opens the translation WebSocket, streams 24 kHz PCM
//! chunks, and turns the transcript-delta stream into caption events. Auto-reconnects on drop.
//!
//! Two things differ from the Gemini client: authentication is an `Authorization: Bearer`
//! header (not a query param), and the translate stream has **no turn-complete event** — so a
//! caption is finalized after a short idle gap with no new translated text (`FINALIZE_AFTER`).
//! See `docs/openai-realtime-api.md`.

use std::time::Duration;

use anyhow::{Context, Result};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{header::AUTHORIZATION, HeaderValue};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

use super::protocol::{InputAudioAppend, ServerEvent, SessionUpdate};
use crate::audio::AudioChunk;
use crate::types::{events, Caption, Origin, SessionState, StatusUpdate};

/// Dedicated speech-to-speech translate model (captions come from its transcript sidecar).
pub const DEFAULT_OPENAI_TRANSLATE_MODEL: &str = "gpt-realtime-translate";
/// Streaming STT model used for the source-language transcription (operator monitor).
pub const DEFAULT_OPENAI_TRANSCRIBE_MODEL: &str = "gpt-realtime-whisper";
pub const DEFAULT_OPENAI_HOST: &str = "api.openai.com";

/// The translate stream has no turn lifecycle: finalize a caption once this much time passes
/// with no new transcript text.
const FINALIZE_AFTER: Duration = Duration::from_millis(900);
/// "Disarm" sentinel for the finalize timer (far in the future).
const IDLE: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
    pub transcribe_model: String,
    pub host: String,
    pub target_language_code: String,
    pub origin: Origin,
}

impl OpenAiConfig {
    fn ws_url(&self) -> String {
        format!(
            "wss://{}/v1/realtime/translations?model={}",
            self.host, self.model
        )
    }
}

/// Accumulates transcript deltas for the current (implicit) turn.
#[derive(Default)]
struct TurnAccumulator {
    id: u64,
    source: String,
    translated: String,
}

/// Run a translation session for one audio source, reconnecting until cancelled.
pub async fn run_session(
    app: AppHandle,
    cfg: OpenAiConfig,
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
                if cancel.is_cancelled() {
                    break;
                }
                tracing::warn!(origin = ?cfg.origin, "OpenAI stream closed; reconnecting");
            }
            Err(e) => {
                tracing::error!(origin = ?cfg.origin, "OpenAI stream error: {e:#}");
                emit_status(&app, SessionState::Reconnecting, Some(format!("{e}")));
            }
        }

        first = false;
        if cancel.is_cancelled() {
            break;
        }

        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = tokio::time::sleep(backoff) => {}
        }
        backoff = (backoff * 2).min(Duration::from_secs(16));
    }

    tracing::info!(origin = ?cfg.origin, "OpenAI session loop ended");
}

async fn connect_and_run(
    app: &AppHandle,
    cfg: &OpenAiConfig,
    audio_rx: &mut UnboundedReceiver<AudioChunk>,
    cancel: &CancellationToken,
) -> Result<()> {
    // OpenAI authenticates the WebSocket with an Authorization header, not a query param.
    let mut request = cfg
        .ws_url()
        .into_client_request()
        .context("failed to build OpenAI request")?;
    let bearer = HeaderValue::from_str(&format!("Bearer {}", cfg.api_key))
        .context("OpenAI API key is not a valid header value")?;
    request.headers_mut().insert(AUTHORIZATION, bearer);

    let (ws, _resp) = connect_async(request)
        .await
        .context("WebSocket connect failed")?;
    let (mut write, mut read) = ws.split();

    // Configure the session: target output language + source transcription.
    let setup = SessionUpdate::translate(&cfg.target_language_code, &cfg.transcribe_model);
    write
        .send(Message::Text(serde_json::to_string(&setup)?))
        .await
        .context("failed to send session.update")?;

    tracing::info!(origin = ?cfg.origin, target = %cfg.target_language_code, "connected to OpenAI Realtime translate");
    emit_status(app, SessionState::Running, None);

    let mut acc = TurnAccumulator::default();

    // Finalize timer: armed (now + FINALIZE_AFTER) on transcript activity, disarmed otherwise.
    let finalize = tokio::time::sleep(IDLE);
    tokio::pin!(finalize);

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
                        let msg = InputAudioAppend::pcm16(data);
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
                        if handle_event(app, cfg.origin, &text, &mut acc) {
                            finalize.as_mut().reset(tokio::time::Instant::now() + FINALIZE_AFTER);
                        }
                    }
                    Some(Ok(Message::Binary(bytes))) => {
                        // The server occasionally frames JSON as binary.
                        if let Ok(text) = String::from_utf8(bytes) {
                            if handle_event(app, cfg.origin, &text, &mut acc) {
                                finalize.as_mut().reset(tokio::time::Instant::now() + FINALIZE_AFTER);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => return Ok(()),
                    Some(Ok(_)) => { /* ping/pong handled by the library */ }
                    Some(Err(e)) => return Err(e).context("WebSocket read error"),
                }
            }

            _ = &mut finalize => {
                // Idle gap with no new transcript: close out the current caption.
                if !acc.translated.is_empty() || !acc.source.is_empty() {
                    emit_caption(app, cfg.origin, &acc, true);
                    acc.id += 1;
                    acc.source.clear();
                    acc.translated.clear();
                }
                finalize.as_mut().reset(tokio::time::Instant::now() + IDLE);
            }
        }
    }
}

/// Apply one server event. Returns `true` if it added transcript text, so the caller re-arms
/// the finalize timer.
fn handle_event(app: &AppHandle, origin: Origin, text: &str, acc: &mut TurnAccumulator) -> bool {
    let ev: ServerEvent = match serde_json::from_str(text) {
        Ok(e) => e,
        Err(e) => {
            tracing::debug!("unparsed OpenAI event: {e} :: {text}");
            return false;
        }
    };

    if ev.error.is_some() {
        tracing::warn!(?origin, "OpenAI error event: {text}");
        return false;
    }

    let kind = ev.kind.as_str();

    if kind.ends_with("input_transcript.delta") {
        if let Some(t) = ev.payload() {
            acc.source.push_str(t);
            emit_caption(app, origin, acc, false);
            return true;
        }
    } else if kind.ends_with("output_transcript.delta") {
        if let Some(t) = ev.payload() {
            acc.translated.push_str(t);
            emit_caption(app, origin, acc, false);
            return true;
        }
    } else if kind.ends_with("output_transcript.done") || kind.ends_with("output_transcript.completed")
    {
        // Some preview builds send an explicit completion; finalize immediately.
        if acc.translated.is_empty() {
            if let Some(t) = ev.transcript.as_deref() {
                acc.translated.push_str(t);
            }
        }
        emit_caption(app, origin, acc, true);
        acc.id += 1;
        acc.source.clear();
        acc.translated.clear();
    }

    false
}

fn emit_caption(app: &AppHandle, origin: Origin, acc: &TurnAccumulator, final_: bool) {
    let caption = Caption {
        turn_id: acc.id,
        text: acc.translated.clone(),
        source_text: acc.source.clone(),
        final_,
        origin,
    };
    let _ = app.emit(events::CAPTION, caption.to_json());
}

fn emit_status(app: &AppHandle, state: SessionState, message: Option<String>) {
    let _ = app.emit(events::STATUS, StatusUpdate { state, message });
}
