//! Mistral Voxtral Mini realtime transcription. Unlike the translation providers, its
//! transcript is the audience caption itself, so it is stored in `translated` and exported
//! through the existing caption/transcript path.

use std::time::Duration;

use anyhow::{Context, Result};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::{header::AUTHORIZATION, HeaderValue};

use super::protocol::{InputAudioAppend, ServerEvent, SessionUpdate};
use crate::realtime::{
    CaptionUpdate, MessageControl, MessageOutcome, RealtimeProtocol, TurnAccumulator,
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

    fn handle_message(&mut self, text: &str, acc: &mut TurnAccumulator) -> MessageOutcome {
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
                    return MessageOutcome::activity(CaptionUpdate::Interim);
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
                let caption = if acc.is_empty() {
                    CaptionUpdate::None
                } else {
                    CaptionUpdate::Final
                };
                return MessageOutcome {
                    caption,
                    control: MessageControl::Closed,
                    ..MessageOutcome::default()
                };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realtime::test_support::{Emitted, Harness};

    fn harness() -> Harness<MistralConfig> {
        Harness::new(MistralConfig {
            api_key: String::new(),
            model: DEFAULT_MISTRAL_MODEL.to_string(),
            host: DEFAULT_MISTRAL_HOST.to_string(),
            target_streaming_delay_ms: DEFAULT_TARGET_STREAMING_DELAY_MS,
            origin: Origin::Microphone,
            received_delta: false,
        })
    }

    #[test]
    fn deltas_are_interim_activity() {
        let mut h = harness();
        let outcome = h.send(r#"{"type":"transcription.text.delta","text":"Hello "}"#);
        assert_eq!(outcome.caption, CaptionUpdate::Interim);
        assert!(outcome.transcript_activity);
        h.send(r#"{"type":"transcription.text.delta","text":""}"#);
        h.send(r#"{"type":"transcription.text.delta","text":"world"}"#);
        assert_eq!(
            h.captions,
            [
                Emitted::interim(0, "Hello ", ""),
                Emitted::interim(0, "Hello world", ""),
            ]
        );
    }

    /// `done` repeats the whole session. After deltas, idle finalization has already
    /// committed those turns, so replaying `done.text` would print every line twice.
    #[test]
    fn done_after_deltas_does_not_replay_the_session() {
        let mut h = harness();
        h.send(r#"{"type":"transcription.text.delta","text":"First line."}"#);
        h.finalize_idle();
        let outcome = h.send(r#"{"type":"transcription.done","text":"First line."}"#);
        assert!(matches!(outcome.control, MessageControl::Closed));
        assert_eq!(
            h.captions,
            [
                Emitted::interim(0, "First line.", ""),
                Emitted::final_(0, "First line.", ""),
            ]
        );
    }

    #[test]
    fn done_without_deltas_supplies_the_whole_transcript() {
        let mut h = harness();
        h.send(r#"{"type":"session.created"}"#);
        h.send(r#"{"type":"transcription.done","text":"Only the summary."}"#);
        assert_eq!(h.captions, [Emitted::final_(0, "Only the summary.", "")]);
    }

    #[test]
    fn a_new_session_forgets_earlier_deltas() {
        let mut h = harness();
        h.send(r#"{"type":"transcription.text.delta","text":"Before reconnect"}"#);
        h.finalize_idle();
        h.send(r#"{"type":"session.updated"}"#);
        h.send(r#"{"type":"transcription.done","text":"After reconnect"}"#);
        assert_eq!(
            h.captions.last(),
            Some(&Emitted::final_(1, "After reconnect", ""))
        );
    }

    #[test]
    fn errors_are_fatal() {
        let mut h = harness();
        assert!(matches!(
            h.send(r#"{"type":"error","error":{"message":"invalid key"}}"#)
                .control,
            MessageControl::Fatal(_)
        ));
    }
}
