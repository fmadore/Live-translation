//! Wire types for OpenAI's Realtime *translations* WebSocket
//! (`/v1/realtime/translations`, model `gpt-realtime-translate`).
//! See `docs/openai-realtime-api.md` for the verified schema and references.

use serde::{Deserialize, Serialize};

use crate::types::Origin;

// ---- Outgoing ------------------------------------------------------------

/// `session.update` — sent once after connect to set the target output language and turn on
/// source-language transcription. This is the translate model's only configuration knob; the
/// source language is auto-detected.
#[derive(Debug, Serialize)]
pub struct SessionUpdate {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub session: SessionConfig,
}

#[derive(Debug, Serialize)]
pub struct SessionConfig {
    pub audio: AudioConfig,
}

#[derive(Debug, Serialize)]
pub struct AudioConfig {
    pub input: AudioInput,
    pub output: AudioOutput,
}

#[derive(Debug, Serialize)]
pub struct AudioInput {
    /// Source-language transcription model — drives the operator monitor.
    pub transcription: TranscriptionModel,
    /// Written as `null` when off, never left out: the reference says `null` disables the
    /// filter and does not say what a missing field means.
    pub noise_reduction: Option<NoiseReduction>,
}

#[derive(Debug, Serialize)]
pub struct TranscriptionModel {
    pub model: String,
}

/// The input noise filter. The server echoes it with a `strength` of 1, its default, which the
/// app leaves alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NoiseReduction {
    #[serde(rename = "type")]
    pub kind: &'static str,
}

impl NoiseReduction {
    /// For close-talking microphones such as headphones. The app cannot tell a headset from a
    /// room microphone, so it never picks this; the live comparison does.
    #[cfg(test)]
    pub const NEAR_FIELD: Self = Self { kind: "near_field" };
    /// For far-field microphones such as laptop or conference-room ones.
    pub const FAR_FIELD: Self = Self { kind: "far_field" };

    /// The filter for a capture source (issue #249, `docs/openai-realtime-api.md`).
    ///
    /// - The microphone gets `far_field`. In a hall it is a laptop, lectern or room
    ///   microphone at a distance from the speaker, and a close-talking filter risks treating
    ///   a distant voice as the background it removes. A headset loses little to it.
    /// - System audio gets none. It is already digital and, from a meeting, already cleaned
    ///   by the far end; a filter tuned for microphones can only take something away.
    pub fn for_origin(origin: Origin) -> Option<Self> {
        match origin {
            Origin::Microphone => Some(Self::FAR_FIELD),
            Origin::System => None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AudioOutput {
    /// BCP-47 target language code; the model auto-detects the source.
    pub language: String,
}

impl SessionUpdate {
    pub fn translate(
        target_language_code: &str,
        transcription_model: &str,
        noise_reduction: Option<NoiseReduction>,
    ) -> Self {
        SessionUpdate {
            kind: "session.update",
            session: SessionConfig {
                audio: AudioConfig {
                    input: AudioInput {
                        transcription: TranscriptionModel {
                            model: transcription_model.to_string(),
                        },
                        noise_reduction,
                    },
                    output: AudioOutput {
                        language: target_language_code.to_string(),
                    },
                },
            },
        }
    }
}

/// `session.input_audio_buffer.append` — one base64 PCM-16 (24 kHz mono) chunk.
#[derive(Debug, Serialize)]
pub struct InputAudioAppend {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub audio: String,
}

impl InputAudioAppend {
    pub fn pcm16(base64_data: String) -> Self {
        InputAudioAppend {
            kind: "session.input_audio_buffer.append",
            audio: base64_data,
        }
    }
}

// ---- Incoming ------------------------------------------------------------

/// A server event. The translations stream has no turn lifecycle, so we read the transcript
/// deltas and finalize captions on a short idle gap (see the client). We match on the `type`
/// *suffix* to stay robust to the `session.`/`response.` prefix the preview API uses.
#[derive(Debug, Default, Deserialize)]
pub struct ServerEvent {
    #[serde(rename = "type", default)]
    pub kind: String,
    /// Incremental transcript text on `*output_transcript.delta` / `*input_transcript.delta`.
    #[serde(default)]
    pub delta: Option<String>,
    /// Full transcript on a `*.done` / `*.completed` event, if the server sends one.
    #[serde(default)]
    pub transcript: Option<String>,
    /// Some event variants carry the text as `text` rather than `delta`.
    #[serde(default)]
    pub text: Option<String>,
    /// Present on `error` events.
    #[serde(default)]
    pub error: Option<serde_json::Value>,
}

impl ServerEvent {
    /// Best-effort text payload (delta first, then text, then full transcript).
    pub fn payload(&self) -> Option<&str> {
        self.delta
            .as_deref()
            .or(self.text.as_deref())
            .or(self.transcript.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_update_sets_language_and_transcription() {
        let v = serde_json::to_value(SessionUpdate::translate(
            "fr",
            "gpt-realtime-whisper",
            Some(NoiseReduction::FAR_FIELD),
        ))
        .unwrap();
        assert_eq!(v["type"], "session.update");
        assert_eq!(v["session"]["audio"]["output"]["language"], "fr");
        assert_eq!(
            v["session"]["audio"]["input"]["transcription"]["model"],
            "gpt-realtime-whisper"
        );
        assert_eq!(
            v["session"]["audio"]["input"]["noise_reduction"]["type"],
            "far_field"
        );
    }

    /// The server's answers to the two updates the client sends, captured verbatim by
    /// `live_probe::noise_reduction_by_value` on 23 November 2026.
    const UPDATED_FAR_FIELD: &str = r#"{"event_id":"event_EnIVgcGiqFOB0avcTIHIO","session":{"audio":{"input":{"noise_reduction":{"strength":1.0,"type":"far_field"},"transcription":{"model":"gpt-realtime-whisper"}},"output":{"language":"en"}},"expires_at":1795421532,"id":"sess_EnIVfT8o12ixTNvD9jnHV","include":null,"model":"gpt-realtime-translate","type":"translation"},"type":"session.updated"}"#;
    const UPDATED_OFF: &str = r#"{"event_id":"event_EnIWCrnglsgeoEEE3ITbx","session":{"audio":{"input":{"noise_reduction":null,"transcription":{"model":"gpt-realtime-whisper"}},"output":{"language":"en"}},"expires_at":1795421564,"id":"sess_EnIWCjIWvtOIbaSvrwaSs","include":null,"model":"gpt-realtime-translate","type":"translation"},"type":"session.updated"}"#;

    /// The microphone is filtered as a far-field source and system audio not at all, written
    /// as an explicit `null`; and the server took each, echoing the same value back.
    #[test]
    fn each_source_gets_the_noise_filter_the_server_accepted() {
        for (origin, recorded) in [
            (Origin::Microphone, UPDATED_FAR_FIELD),
            (Origin::System, UPDATED_OFF),
        ] {
            let sent = serde_json::to_value(SessionUpdate::translate(
                "en",
                "gpt-realtime-whisper",
                NoiseReduction::for_origin(origin),
            ))
            .unwrap();
            let sent = &sent["session"]["audio"]["input"];
            assert!(
                sent.as_object().unwrap().contains_key("noise_reduction"),
                "{origin:?}: off is written as null, not left out"
            );
            let echoed: serde_json::Value = serde_json::from_str(recorded).unwrap();
            assert_eq!(echoed["type"], "session.updated");
            let echoed = &echoed["session"]["audio"]["input"]["noise_reduction"];
            assert_eq!(
                sent["noise_reduction"]["type"], echoed["type"],
                "{origin:?}"
            );
            assert_eq!(sent["noise_reduction"].is_null(), echoed.is_null());
            // The answer is not a caption or an error; the client reads past it.
            let event: ServerEvent = serde_json::from_str(recorded).unwrap();
            assert!(event.error.is_none() && event.payload().is_none());
        }
    }

    #[test]
    fn audio_append_carries_base64_and_type() {
        let v = serde_json::to_value(InputAudioAppend::pcm16("AAA=".into())).unwrap();
        assert_eq!(v["type"], "session.input_audio_buffer.append");
        assert_eq!(v["audio"], "AAA=");
    }

    #[test]
    fn parses_output_transcript_delta() {
        let raw = r#"{"type":"session.output_transcript.delta","delta":"Bonjour"}"#;
        let ev: ServerEvent = serde_json::from_str(raw).unwrap();
        assert!(ev.kind.ends_with("output_transcript.delta"));
        assert_eq!(ev.payload(), Some("Bonjour"));
    }

    #[test]
    fn payload_falls_back_to_transcript_on_done() {
        let raw = r#"{"type":"session.output_transcript.done","transcript":"Hello world"}"#;
        let ev: ServerEvent = serde_json::from_str(raw).unwrap();
        assert!(ev.kind.ends_with("output_transcript.done"));
        assert_eq!(ev.payload(), Some("Hello world"));
    }
}
