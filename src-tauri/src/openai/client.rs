//! OpenAI Realtime *translations* protocol: connection details and event handling for the
//! shared realtime session runner (`crate::realtime`). Streams 24 kHz PCM chunks and turns
//! the transcript-delta stream into caption events.
//!
//! Two things differ from Gemini: authentication is an `Authorization: Bearer` header (not
//! a query param), and the translate stream has **no turn-complete event** — so a caption
//! is finalized after a short idle gap with no new translated text (`FINALIZE_AFTER`, run
//! by the shared runner's idle timer). See `docs/openai-realtime-api.md`.

use std::time::Duration;

use anyhow::{Context, Result};
use tauri::AppHandle;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::{header::AUTHORIZATION, HeaderValue};

use super::protocol::{InputAudioAppend, ServerEvent, SessionUpdate};
use crate::realtime::{emit_caption, RealtimeProtocol, TurnAccumulator};
use crate::types::Origin;

/// Dedicated speech-to-speech translate model (captions come from its transcript sidecar).
pub const DEFAULT_OPENAI_TRANSLATE_MODEL: &str = "gpt-realtime-translate";
/// Streaming STT model used for the source-language transcription (operator monitor).
pub const DEFAULT_OPENAI_TRANSCRIBE_MODEL: &str = "gpt-realtime-whisper";
pub const DEFAULT_OPENAI_HOST: &str = "api.openai.com";

/// The translate stream has no turn lifecycle: finalize a caption once this much time
/// passes with no new transcript text.
const FINALIZE_AFTER: Duration = Duration::from_millis(900);

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

impl RealtimeProtocol for OpenAiConfig {
    const NAME: &'static str = "OpenAI";

    fn origin(&self) -> Origin {
        self.origin
    }

    fn connect_request(&self) -> Result<Request> {
        // OpenAI authenticates the WebSocket with an Authorization header, not a query param.
        let mut request = self
            .ws_url()
            .into_client_request()
            .context("failed to build OpenAI request")?;
        let bearer = HeaderValue::from_str(&format!("Bearer {}", self.api_key))
            .context("OpenAI API key is not a valid header value")?;
        request.headers_mut().insert(AUTHORIZATION, bearer);
        Ok(request)
    }

    fn setup_json(&self) -> Result<String> {
        let setup = SessionUpdate::translate(&self.target_language_code, &self.transcribe_model);
        Ok(serde_json::to_string(&setup)?)
    }

    fn audio_json(&self, base64_pcm: String) -> Result<String> {
        Ok(serde_json::to_string(&InputAudioAppend::pcm16(base64_pcm))?)
    }

    fn handle_message(&mut self, app: &AppHandle, text: &str, acc: &mut TurnAccumulator) -> bool {
        let ev: ServerEvent = match serde_json::from_str(text) {
            Ok(e) => e,
            Err(e) => {
                tracing::debug!("unparsed OpenAI event: {e} :: {text}");
                return false;
            }
        };

        if ev.error.is_some() {
            tracing::warn!(origin = ?self.origin, "OpenAI error event: {text}");
            return false;
        }

        let kind = ev.kind.as_str();

        if kind.ends_with("input_transcript.delta") {
            if let Some(t) = ev.payload() {
                acc.source.push_str(t);
                emit_caption(app, self.origin, acc, false);
                return true;
            }
        } else if kind.ends_with("output_transcript.delta") {
            if let Some(t) = ev.payload() {
                acc.translated.push_str(t);
                emit_caption(app, self.origin, acc, false);
                return true;
            }
        } else if kind.ends_with("output_transcript.done")
            || kind.ends_with("output_transcript.completed")
        {
            // Some preview builds send an explicit completion; finalize immediately.
            if acc.translated.is_empty() {
                if let Some(t) = ev.transcript.as_deref() {
                    acc.translated.push_str(t);
                }
            }
            emit_caption(app, self.origin, acc, true);
            acc.next_turn();
        }

        false
    }

    fn finalize_after(&self) -> Option<Duration> {
        Some(FINALIZE_AFTER)
    }
}
