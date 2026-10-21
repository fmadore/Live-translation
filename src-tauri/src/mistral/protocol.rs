//! Wire types for Mistral's realtime transcription WebSocket.
//! The shapes mirror the official `mistralai` SDK's realtime connection implementation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct SessionUpdate<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub session: SessionConfig<'a>,
}

#[derive(Debug, Serialize)]
pub struct SessionConfig<'a> {
    pub audio_format: AudioFormat,
    pub target_streaming_delay_ms: u32,
    /// Names and terms to spell as written, for the whole session.
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    pub context_bias: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<TurnDetection>,
}

/// Empty, it turns end-of-turn detection on with Mistral's default thresholds.
#[derive(Debug, Serialize)]
pub struct TurnDetection {}

#[derive(Debug, Serialize)]
pub struct AudioFormat {
    pub encoding: &'static str,
    pub sample_rate: u32,
}

impl<'a> SessionUpdate<'a> {
    /// 16 kHz PCM16 at this delay, and nothing else: all that Realtime 2602 accepts.
    pub fn pcm16(target_streaming_delay_ms: u32) -> Self {
        Self {
            kind: "session.update",
            session: SessionConfig {
                audio_format: AudioFormat {
                    encoding: "pcm_s16le",
                    sample_rate: 16_000,
                },
                target_streaming_delay_ms,
                context_bias: &[],
                turn_detection: None,
            },
        }
    }

    /// Realtime 3's additions: end-of-turn detection, and the session's glossary if it has
    /// one. 2602 answers either with an error that ends the session.
    pub fn with_turns_and_bias(mut self, context_bias: &'a [String]) -> Self {
        self.session.context_bias = context_bias;
        self.session.turn_detection = Some(TurnDetection {});
        self
    }
}

#[derive(Debug, Serialize)]
pub struct InputAudioAppend {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub audio: String,
}

impl InputAudioAppend {
    pub fn pcm16(audio: String) -> Self {
        Self {
            kind: "input_audio.append",
            audio,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ServerEvent {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub error: Option<serde_json::Value>,
    /// On `turn.end`: `threshold` when the model judged the turn over, `timeout` when the
    /// pause simply ran long.
    #[serde(default)]
    pub end_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_matches_official_realtime_schema() {
        let value = serde_json::to_value(SessionUpdate::pcm16(480)).unwrap();
        assert_eq!(value["type"], "session.update");
        assert_eq!(value["session"]["audio_format"]["encoding"], "pcm_s16le");
        assert_eq!(value["session"]["audio_format"]["sample_rate"], 16_000);
        assert_eq!(value["session"]["target_streaming_delay_ms"], 480);
        assert_eq!(value["session"].as_object().unwrap().len(), 2);
    }

    #[test]
    fn realtime_3_setup_turns_on_turn_detection_and_sends_any_glossary() {
        let terms = ["Voxtral".to_string(), "Alder Creek Design".to_string()];
        let value =
            serde_json::to_value(SessionUpdate::pcm16(480).with_turns_and_bias(&terms)).unwrap();
        assert_eq!(value["session"]["turn_detection"], serde_json::json!({}));
        assert_eq!(
            value["session"]["context_bias"],
            serde_json::json!(["Voxtral", "Alder Creek Design"])
        );

        let value =
            serde_json::to_value(SessionUpdate::pcm16(480).with_turns_and_bias(&[])).unwrap();
        assert_eq!(value["session"]["turn_detection"], serde_json::json!({}));
        assert!(value["session"].get("context_bias").is_none());
    }

    #[test]
    fn audio_append_matches_official_realtime_schema() {
        let value = serde_json::to_value(InputAudioAppend::pcm16("AAA=".into())).unwrap();
        assert_eq!(value["type"], "input_audio.append");
        assert_eq!(value["audio"], "AAA=");
    }

    #[test]
    fn parses_text_delta() {
        let event: ServerEvent =
            serde_json::from_str(r#"{"type":"transcription.text.delta","text":"Bonjour"}"#)
                .unwrap();
        assert_eq!(event.kind, "transcription.text.delta");
        assert_eq!(event.text.as_deref(), Some("Bonjour"));
    }
}
