//! Mistral Voxtral Mini realtime transcription. Unlike the translation providers, its
//! transcript is the audience caption itself, so it is stored in `translated` and exported
//! through the existing caption/transcript path.

use std::time::Duration;

use anyhow::Result;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use super::protocol::{InputAudioAppend, ServerEvent, SessionUpdate};
use crate::realtime::{
    bearer_request, CaptionUpdate, MessageControl, MessageOutcome, RealtimeProtocol,
    TurnAccumulator,
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
        bearer_request(Self::NAME, &self.ws_url(), &self.api_key)
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
            // The drain's end after `input_audio.end`. Mid-stream the final caption still goes
            // out, and the runner moves to a new session rather than ending the source.
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
                if event.error.as_ref().is_some_and(recoverable) {
                    tracing::warn!(
                        origin = ?self.origin,
                        error = ?event.error,
                        "Mistral reported a transient error; reconnecting"
                    );
                    return MessageOutcome::control(MessageControl::Reconnect);
                }
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

/// Whether an `error` event is one a new session can get past. The official SDK types it as
/// `{ message, code }`: `message` a string or an object, `code` an integer it calls an
/// internal code for debugging. Codes in HTTP's transient range (timeout, rate limit, server
/// errors) reconnect, as do the transient `type`/`code` names Mistral's HTTP API uses, should
/// `message` be such an object. Everything else — authentication, permission, a bad request
/// or model, an exhausted quota, an unknown code — stays fatal, so a persistent failure
/// reports itself instead of reconnecting in a loop.
fn recoverable(error: &serde_json::Value) -> bool {
    let code = error
        .get("code")
        .and_then(|code| code.as_u64().or_else(|| code.as_str()?.parse().ok()));
    let message = error.get("message");
    let names: Vec<&str> = [error, message.unwrap_or(&serde_json::Value::Null)]
        .into_iter()
        .flat_map(|value| ["type", "code"].map(|key| value.get(key)))
        .filter_map(|name| name?.as_str())
        .collect();
    let text = message.map(|message| message.to_string().to_lowercase());
    // A 429 is also how an exhausted quota or plan limit is reported; no reconnect fixes that.
    if text.is_some_and(|text| {
        ["quota", "billing", "payment", "credit"]
            .iter()
            .any(|w| text.contains(w))
    }) {
        return false;
    }
    matches!(code, Some(408 | 429 | 500 | 502 | 503 | 504))
        || names.iter().any(|name| {
            matches!(
                *name,
                "server_error"
                    | "rate_limit_error"
                    | "rate_limited"
                    | "timeout"
                    | "session_expired"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realtime::test_support::{handshake, Emitted, Harness};

    fn config(api_key: &str) -> MistralConfig {
        MistralConfig {
            api_key: api_key.to_string(),
            model: DEFAULT_MISTRAL_MODEL.to_string(),
            host: DEFAULT_MISTRAL_HOST.to_string(),
            target_streaming_delay_ms: DEFAULT_TARGET_STREAMING_DELAY_MS,
            origin: Origin::Microphone,
            received_delta: false,
        }
    }

    fn harness() -> Harness<MistralConfig> {
        Harness::new(config(""))
    }

    #[test]
    fn the_handshake_authenticates_with_a_bearer_header() {
        let request = config("mistral-test").connect_request().unwrap();
        assert_eq!(
            handshake(request),
            "wss://api.mistral.ai/v1/audio/transcriptions/realtime?model=voxtral-mini-transcribe-realtime-2602\n\
             GET /v1/audio/transcriptions/realtime?model=voxtral-mini-transcribe-realtime-2602 HTTP/1.1\r\n\
             Host: api.mistral.ai\r\n\
             Connection: Upgrade\r\n\
             Upgrade: websocket\r\n\
             Sec-WebSocket-Version: 13\r\n\
             Sec-WebSocket-Key: <key>\r\n\
             authorization: Bearer mistral-test\r\n\
             \r\n"
        );
    }

    /// The operator reads these as the reconnecting status's detail.
    #[test]
    fn a_request_that_cannot_be_built_says_which_part_failed() {
        let error = config("bad\nkey").connect_request().unwrap_err();
        assert_eq!(
            error.to_string(),
            "Mistral API key is not a valid header value"
        );
        let mut config = config("mistral-test");
        config.host = "not a host".to_string();
        let error = config.connect_request().unwrap_err();
        assert_eq!(error.to_string(), "failed to build Mistral request");
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
    fn transient_errors_reconnect_and_the_rest_stop_the_source() {
        let mut h = harness();
        for (error, reconnects) in [
            (r#"{"message":"Internal server error","code":500}"#, true),
            (r#"{"message":"Service unavailable","code":503}"#, true),
            (r#"{"message":"Rate limit exceeded","code":429}"#, true),
            (
                r#"{"message":{"type":"server_error","detail":"retry"},"code":3000}"#,
                true,
            ),
            (r#"{"message":"Monthly quota exceeded","code":429}"#, false),
            (r#"{"message":"Unauthorized","code":401}"#, false),
            (r#"{"message":"Forbidden","code":403}"#, false),
            (
                r#"{"message":"Invalid model: voxtral-unknown","code":400}"#,
                false,
            ),
            (
                r#"{"message":{"detail":"something new"},"code":3001}"#,
                false,
            ),
            (r#"{"message":"invalid key"}"#, false),
        ] {
            let frame = format!(r#"{{"type":"error","error":{error}}}"#);
            let control = h.send(&frame).control;
            assert_eq!(
                matches!(control, MessageControl::Reconnect),
                reconnects,
                "{error} -> {control:?}"
            );
            if !reconnects {
                assert!(matches!(control, MessageControl::Fatal(_)), "{error}");
            }
        }
        assert!(matches!(
            h.send(r#"{"type":"error"}"#).control,
            MessageControl::Fatal(_)
        ));
    }
}
