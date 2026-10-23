//! Gemini Live integration: WebSocket clients and wire protocol. Two models share the one
//! endpoint — Live Translate for translated captions, Transcribe Live for subtitles.

pub mod client;
pub mod protocol;
pub mod transcribe;

use std::time::Duration;

use anyhow::{Context, Result};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use crate::realtime::{classify_json, MessageControl, MessageOutcome, RealtimeProtocol};
use protocol::ServerMessage;

pub use client::{GeminiConfig, DEFAULT_HOST, DEFAULT_TRANSLATE_MODEL};
pub use transcribe::{GeminiTranscribeConfig, DEFAULT_TRANSCRIBE_MODEL};

/// The handshake both models share: one endpoint, authenticated by a `key` query parameter
/// rather than a header; the model is named in the setup message. `provider` names the client
/// in the error, which the operator reads as the detail of a reconnecting status.
fn gemini_request(provider: &str, host: &str, api_key: &str) -> Result<Request> {
    format!(
        "wss://{host}/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key={api_key}"
    )
    .into_client_request()
    .with_context(|| format!("failed to build {provider} request"))
}

impl ServerMessage {
    /// What both models do with a frame before its content: confirm setup, move on `goAway`,
    /// or act on an error as the shared `classify_json` reads it — a `google.rpc.Status` whose
    /// UNAVAILABLE or INTERNAL reconnects, and whose bad request, key or quota stops the source.
    /// `None` leaves the frame's content to the caller. `error_label` is the client's own
    /// wording for a fatal error, which the operator reads after its name.
    fn control<P: RealtimeProtocol>(&self, proto: &P, error_label: &str) -> Option<MessageOutcome> {
        if self.setup_complete.is_some() {
            let origin = proto.origin();
            tracing::debug!(?origin, "{} setup complete; streaming audio", P::NAME);
            return Some(MessageOutcome::setup_complete());
        }
        // Live sessions are capped — transcription at ten minutes, so a long room session
        // reconnects several times an hour. `goAway` warns ahead of the cut, with the time left
        // before it, and as a planned handover the runner drains the old socket within that
        // time, then reconnects at once rather than backing off.
        if let Some(go_away) = &self.go_away {
            let time_left = go_away
                .get("timeLeft")
                .and_then(serde_json::Value::as_str)
                .and_then(duration);
            return Some(MessageOutcome::control(MessageControl::Handover {
                time_left,
            }));
        }
        self.error.as_ref().map(|error| {
            let control = classify_json(error).control(|| format!("{error_label}: {error}"));
            if !matches!(control, MessageControl::Fatal(_)) {
                let origin = proto.origin();
                tracing::warn!(?origin, %error, "{} reported a transient error", P::NAME);
            }
            MessageOutcome::control(control)
        })
    }
}

/// A `google.protobuf.Duration` as JSON writes it: seconds with up to nine decimals and an
/// `s`, such as `"10s"` or `"0.5s"`. `None` for anything else, which leaves the drain its own
/// limit.
fn duration(text: &str) -> Option<Duration> {
    let seconds: f64 = text.strip_suffix('s')?.parse().ok()?;
    Duration::try_from_secs_f64(seconds).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_away_says_how_long_the_connection_has_left() {
        let mut h = Harness::new(GeminiTranscribeConfig {
            api_key: String::new(),
            model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
            host: DEFAULT_HOST.to_string(),
            origin: Origin::Microphone,
        });
        for (frame, expected) in [
            (
                r#"{"goAway":{"timeLeft":"10s"}}"#,
                Some(Duration::from_secs(10)),
            ),
            (
                r#"{"goAway":{"timeLeft":"0.5s"}}"#,
                Some(Duration::from_millis(500)),
            ),
            (r#"{"goAway":{"timeLeft":"soon"}}"#, None),
            (r#"{"goAway":{"timeLeft":"-1s"}}"#, None),
            (r#"{"goAway":{}}"#, None),
        ] {
            let MessageControl::Handover { time_left } = h.send(frame).control else {
                panic!("{frame} did not hand over");
            };
            assert_eq!(time_left, expected, "{frame}");
        }
    }
    use crate::realtime::test_support::Harness;
    use crate::types::Origin;

    fn fatal(outcome: MessageOutcome) -> String {
        match outcome.control {
            MessageControl::Fatal(message) => message,
            other => panic!("expected a fatal outcome, got {other:?}"),
        }
    }

    /// The operator reads this after the provider's name, so each client keeps its own words.
    #[test]
    fn a_provider_error_is_fatal_in_each_clients_words() {
        let frame = r#"{"error":{"code":400,"message":"bad"}}"#;
        let mut translate = Harness::new(GeminiConfig {
            api_key: String::new(),
            model: DEFAULT_TRANSLATE_MODEL.to_string(),
            host: DEFAULT_HOST.to_string(),
            target_language_code: "fr".to_string(),
            origin: Origin::Microphone,
        });
        assert_eq!(
            fatal(translate.send(frame)),
            r#"Gemini realtime error: {"code":400,"message":"bad"}"#
        );
        let mut transcribe = Harness::new(GeminiTranscribeConfig {
            api_key: String::new(),
            model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
            host: DEFAULT_HOST.to_string(),
            origin: Origin::Microphone,
        });
        assert_eq!(
            fatal(transcribe.send(frame)),
            r#"Gemini Transcribe error: {"code":400,"message":"bad"}"#
        );
    }

    /// Every `error` used to be fatal, so one "try again later" at a ten-minute handover left
    /// the source in Error until the operator restarted it.
    #[test]
    fn a_transient_provider_error_reconnects_instead() {
        let mut transcribe = Harness::new(GeminiTranscribeConfig {
            api_key: String::new(),
            model: DEFAULT_TRANSCRIBE_MODEL.to_string(),
            host: DEFAULT_HOST.to_string(),
            origin: Origin::Microphone,
        });
        for frame in [
            r#"{"error":{"code":503,"message":"The service is currently unavailable.","status":"UNAVAILABLE"}}"#,
            r#"{"error":{"code":500,"message":"Internal error encountered.","status":"INTERNAL"}}"#,
        ] {
            assert!(
                matches!(transcribe.send(frame).control, MessageControl::Reconnect),
                "{frame}"
            );
        }
        let quota = r#"{"error":{"code":429,"message":"You exceeded your current quota.","status":"RESOURCE_EXHAUSTED"}}"#;
        assert!(fatal(transcribe.send(quota)).starts_with("Gemini Transcribe error: "));
    }
}
