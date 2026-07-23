//! Gemini Live protocol: connection details and server-message handling for the shared
//! realtime session runner (`crate::realtime`). Sends 16 kHz PCM chunks and turns the
//! returned transcriptions into caption events.

use anyhow::{Context, Result};
use tauri::AppHandle;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use super::protocol::{RealtimeInputMessage, ServerMessage, SetupMessage};
use crate::realtime::{emit_caption, RealtimeProtocol, TurnAccumulator};
use crate::types::Origin;

/// Dedicated speech-to-speech translate model, run in TEXT mode (no audio synthesized).
pub const DEFAULT_TRANSLATE_MODEL: &str = "gemini-3.5-live-translate-preview";
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

impl RealtimeProtocol for GeminiConfig {
    const NAME: &'static str = "Gemini";

    fn origin(&self) -> Origin {
        self.origin
    }

    fn connect_request(&self) -> Result<Request> {
        self.ws_url()
            .into_client_request()
            .context("failed to build Gemini request")
    }

    fn setup_json(&self) -> Result<String> {
        let setup = SetupMessage::live_translate(&self.model, &self.target_language_code);
        Ok(serde_json::to_string(&setup)?)
    }

    fn audio_json(&self, base64_pcm: String) -> Result<String> {
        Ok(serde_json::to_string(&RealtimeInputMessage::pcm16(
            base64_pcm,
        ))?)
    }

    fn handle_message(&mut self, app: &AppHandle, text: &str, acc: &mut TurnAccumulator) -> bool {
        let msg: ServerMessage = match serde_json::from_str(text) {
            Ok(m) => m,
            Err(e) => {
                tracing::debug!("unparsed server message: {e} :: {text}");
                return false;
            }
        };

        if msg.setup_complete.is_some() {
            tracing::debug!(origin = ?self.origin, "Gemini setup complete; streaming audio");
        }
        if msg.go_away.is_some() {
            tracing::info!("Gemini sent goAway; will reconnect");
        }

        let Some(content) = msg.server_content else {
            return false;
        };

        // Source text (operator monitor) comes from the input transcription; the
        // translated text from the output-audio transcription sidecar.
        if let Some(t) = &content.input_transcription {
            acc.source.push_str(&t.text);
        }
        let translated_delta = content
            .output_transcription
            .as_ref()
            .map(|t| t.text.as_str());
        let got_translation = translated_delta.is_some_and(|s| !s.is_empty());
        if let Some(delta) = translated_delta {
            acc.translated.push_str(delta);
        }

        let turn_complete = content.turn_complete.unwrap_or(false);

        // Emit whenever we have new text, or to mark the turn final.
        if got_translation || content.input_transcription.is_some() || turn_complete {
            emit_caption(app, self.origin, acc, turn_complete);
        }
        if turn_complete {
            acc.next_turn();
        }

        false // Gemini has a real turn lifecycle; the idle-finalize timer stays disabled.
    }
}
