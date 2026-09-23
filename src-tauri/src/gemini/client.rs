//! Gemini Live protocol: connection details and server-message handling for the shared
//! realtime session runner (`crate::realtime`). Sends 16 kHz PCM chunks and turns the
//! returned transcriptions into caption events.

use anyhow::{Context, Result};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use super::protocol::{RealtimeInputMessage, ServerMessage, SetupMessage};
use crate::realtime::{
    CaptionUpdate, MessageControl, MessageOutcome, RealtimeProtocol, TurnAccumulator,
};
use crate::types::Origin;

/// Dedicated speech-to-speech translate model; audio output is discarded after its
/// transcription sidecar is extracted.
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

    fn wait_for_setup_complete(&self) -> bool {
        true
    }

    fn handle_message(&mut self, text: &str, acc: &mut TurnAccumulator) -> MessageOutcome {
        let msg: ServerMessage = match serde_json::from_str(text) {
            Ok(m) => m,
            Err(e) => {
                tracing::debug!("unparsed server message: {e} :: {text}");
                return MessageOutcome::default();
            }
        };

        if msg.setup_complete.is_some() {
            tracing::debug!(origin = ?self.origin, "Gemini setup complete; streaming audio");
            return MessageOutcome::setup_complete();
        }
        if msg.go_away.is_some() {
            return MessageOutcome::control(MessageControl::Reconnect);
        }
        if let Some(error) = msg.error {
            return MessageOutcome::control(MessageControl::Fatal(format!(
                "Gemini realtime error: {error}"
            )));
        }

        let Some(content) = msg.server_content else {
            return MessageOutcome::default();
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

        // Emit whenever we have new text, or to mark the turn final.
        if content.turn_complete.unwrap_or(false) {
            MessageOutcome::caption(CaptionUpdate::Final)
        } else if got_translation || content.input_transcription.is_some() {
            MessageOutcome::caption(CaptionUpdate::Interim)
        } else {
            MessageOutcome::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realtime::test_support::{Emitted, Harness};

    fn harness() -> Harness<GeminiConfig> {
        Harness::new(GeminiConfig {
            api_key: String::new(),
            model: DEFAULT_TRANSLATE_MODEL.to_string(),
            host: DEFAULT_HOST.to_string(),
            target_language_code: "fr".to_string(),
            origin: Origin::Microphone,
        })
    }

    #[test]
    fn source_and_translation_accumulate_until_the_turn_completes() {
        let mut h = harness();
        h.send(r#"{"serverContent":{"inputTranscription":{"text":"Good "}}}"#);
        h.send(r#"{"serverContent":{"inputTranscription":{"text":"morning"},"outputTranscription":{"text":"Bonjour"}}}"#);
        h.send(r#"{"serverContent":{"turnComplete":true}}"#);
        h.send(r#"{"serverContent":{"outputTranscription":{"text":"Merci"}}}"#);
        assert_eq!(
            h.captions,
            [
                Emitted::interim(0, "", "Good "),
                Emitted::interim(0, "Bonjour", "Good morning"),
                Emitted::final_(0, "Bonjour", "Good morning"),
                Emitted::interim(1, "Merci", ""),
            ]
        );
    }

    #[test]
    fn an_empty_translation_delta_alone_emits_nothing() {
        let mut h = harness();
        h.send(r#"{"serverContent":{"outputTranscription":{"text":""}}}"#);
        h.send(r#"{"serverContent":{"modelTurn":{}}}"#);
        assert!(h.captions.is_empty());
    }

    #[test]
    fn setup_go_away_and_errors_steer_the_connection() {
        let mut h = harness();
        assert!(h.send(r#"{"setupComplete":{}}"#).setup_complete);
        assert!(matches!(
            h.send(r#"{"goAway":{"timeLeft":"10s"}}"#).control,
            MessageControl::Reconnect
        ));
        assert!(matches!(
            h.send(r#"{"error":{"code":400,"message":"bad"}}"#).control,
            MessageControl::Fatal(_)
        ));
        assert!(h.captions.is_empty());
    }
}
