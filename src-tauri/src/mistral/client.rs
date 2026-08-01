//! Mistral Voxtral Mini realtime transcription. Unlike the translation providers, its
//! transcript is the audience caption itself, so it is stored in `translated` and exported
//! through the existing caption/transcript path.

use std::time::Duration;

use anyhow::{Context, Result};
use tauri::AppHandle;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::{header::AUTHORIZATION, HeaderValue};

use super::protocol::{InputAudioAppend, ServerEvent, SessionUpdate};
use crate::realtime::{
    emit_caption, MessageControl, MessageOutcome, RealtimeProtocol, TurnAccumulator,
};
use crate::types::Origin;

pub const DEFAULT_MISTRAL_MODEL: &str = "voxtral-mini-transcribe-realtime-2602";
pub const DEFAULT_MISTRAL_HOST: &str = "api.mistral.ai";
/// A small context window keeps subtitles responsive while improving recognition quality.
pub const DEFAULT_TARGET_STREAMING_DELAY_MS: u32 = 480;
const FINALIZE_AFTER: Duration = Duration::from_millis(900);

#[derive(Clone)]
pub struct MistralConfig {
    pub api_key: String,
    pub model: String,
    pub host: String,
    pub target_streaming_delay_ms: u32,
    pub origin: Origin,
    pub received_delta: bool,
}

impl MistralConfig {
    fn ws_url(&self) -> String {
        format!(
            "wss://{}/v1/audio/transcriptions/realtime?model={}",
            self.host, self.model
        )
    }
}

impl RealtimeProtocol for MistralConfig {
    const NAME: &'static str = "Mistral";

    fn origin(&self) -> Origin {
        self.origin
    }

    fn connect_request(&self) -> Result<Request> {
        let mut request = self
            .ws_url()
            .into_client_request()
            .context("failed to build Mistral request")?;
        let bearer = HeaderValue::from_str(&format!("Bearer {}", self.api_key))
            .context("Mistral API key is not a valid header value")?;
        request.headers_mut().insert(AUTHORIZATION, bearer);
        Ok(request)
    }

    fn setup_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&SessionUpdate::pcm16(
            self.target_streaming_delay_ms,
        ))?)
    }

    fn audio_json(&self, base64_pcm: String) -> Result<String> {
        Ok(serde_json::to_string(&InputAudioAppend::pcm16(base64_pcm))?)
    }

    fn closing_json(&self) -> Result<Vec<String>> {
        Ok(vec![
            r#"{"type":"input_audio.flush"}"#.to_string(),
            r#"{"type":"input_audio.end"}"#.to_string(),
        ])
    }

    fn handle_message(
        &mut self,
        app: &AppHandle,
        text: &str,
        acc: &mut TurnAccumulator,
    ) -> MessageOutcome {
        let event: ServerEvent = match serde_json::from_str(text) {
            Ok(event) => event,
            Err(error) => {
                tracing::debug!("unparsed Mistral event: {error} :: {text}");
                return MessageOutcome::default();
            }
        };

        match event.kind.as_str() {
            "session.created" | "session.updated" => {
                self.received_delta = false;
                tracing::debug!(origin = ?self.origin, event = %event.kind, "Mistral session ready");
            }
            "transcription.text.delta" => {
                if let Some(delta) = event.text.as_deref().filter(|delta| !delta.is_empty()) {
                    self.received_delta = true;
                    acc.translated.push_str(delta);
                    emit_caption(app, self.origin, acc, false);
                    return MessageOutcome::activity();
                }
            }
            "transcription.done" => {
                // `done.text` contains the full session transcript. Only use it when the
                // server sent no deltas; otherwise idle-finalized turns would be duplicated.
                if !self.received_delta && acc.translated.is_empty() {
                    if let Some(full_text) = event.text {
                        acc.translated = full_text;
                    }
                }
                if !acc.is_empty() {
                    emit_caption(app, self.origin, acc, true);
                    acc.next_turn();
                }
                return MessageOutcome::control(MessageControl::Closed);
            }
            "error" => {
                return MessageOutcome::control(MessageControl::Fatal(
                    event
                        .error
                        .map(|error| error.to_string())
                        .unwrap_or_else(|| "Mistral realtime transcription error".to_string()),
                ));
            }
            _ => {}
        }

        MessageOutcome::default()
    }

    fn finalize_after(&self) -> Option<Duration> {
        Some(FINALIZE_AFTER)
    }
}
