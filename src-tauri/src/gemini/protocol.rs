//! Wire types for the Gemini Live `BidiGenerateContent` WebSocket protocol.
//! See `docs/gemini-live-api.md` for the verified schema and references.

use serde::{Deserialize, Serialize};

// ---- Outgoing ------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SetupMessage {
    pub setup: Setup,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Setup {
    pub model: String,
    pub generation_config: GenerationConfig,
    /// Empty object enables transcription of the recognised source speech.
    pub input_audio_transcription: Empty,
    /// Empty object enables transcription of the translated output speech.
    pub output_audio_transcription: Empty,
}

#[derive(Debug, Serialize)]
pub struct Empty {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    pub response_modalities: Vec<String>,
    pub translation_config: TranslationConfig,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationConfig {
    pub target_language_code: String,
    pub echo_target_language: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeInputMessage {
    pub realtime_input: RealtimeInput,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeInput {
    pub media_chunks: Vec<MediaChunk>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaChunk {
    pub mime_type: String,
    pub data: String,
}

impl SetupMessage {
    pub fn new(model: &str, target_language_code: &str) -> Self {
        SetupMessage {
            setup: Setup {
                model: format!("models/{model}"),
                generation_config: GenerationConfig {
                    // The translate model is speech-to-speech; AUDIO is its native
                    // modality. We consume only the transcriptions below.
                    response_modalities: vec!["AUDIO".to_string()],
                    translation_config: TranslationConfig {
                        target_language_code: target_language_code.to_string(),
                        echo_target_language: false,
                    },
                },
                input_audio_transcription: Empty {},
                output_audio_transcription: Empty {},
            },
        }
    }
}

impl RealtimeInputMessage {
    pub fn pcm16(base64_data: String) -> Self {
        RealtimeInputMessage {
            realtime_input: RealtimeInput {
                media_chunks: vec![MediaChunk {
                    mime_type: "audio/pcm;rate=16000".to_string(),
                    data: base64_data,
                }],
            },
        }
    }
}

// ---- Incoming ------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMessage {
    #[serde(default)]
    pub setup_complete: Option<serde_json::Value>,
    #[serde(default)]
    pub server_content: Option<ServerContent>,
    /// Sent shortly before the server closes the connection.
    #[serde(default)]
    pub go_away: Option<serde_json::Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerContent {
    #[serde(default)]
    pub input_transcription: Option<Transcription>,
    #[serde(default)]
    pub output_transcription: Option<Transcription>,
    /// Translated audio parts — parsed for schema completeness but intentionally ignored
    /// (we render captions only and never play the synthesized audio).
    #[serde(default)]
    #[allow(dead_code)]
    pub model_turn: Option<serde_json::Value>,
    #[serde(default)]
    pub turn_complete: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Transcription {
    #[serde(default)]
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_serializes_to_expected_shape() {
        let v = serde_json::to_value(SetupMessage::new("gemini-3.5-live-translate-preview", "en"))
            .unwrap();
        assert_eq!(
            v["setup"]["model"],
            "models/gemini-3.5-live-translate-preview"
        );
        assert_eq!(
            v["setup"]["generationConfig"]["translationConfig"]["targetLanguageCode"],
            "en"
        );
        assert_eq!(
            v["setup"]["generationConfig"]["translationConfig"]["echoTargetLanguage"],
            false
        );
        assert!(v["setup"]["inputAudioTranscription"].is_object());
    }

    #[test]
    fn realtime_input_uses_media_chunks() {
        let v = serde_json::to_value(RealtimeInputMessage::pcm16("AAA=".into())).unwrap();
        assert_eq!(
            v["realtimeInput"]["mediaChunks"][0]["mimeType"],
            "audio/pcm;rate=16000"
        );
        assert_eq!(v["realtimeInput"]["mediaChunks"][0]["data"], "AAA=");
    }

    #[test]
    fn parses_server_content_transcriptions() {
        let raw = r#"{"serverContent":{"outputTranscription":{"text":"Hello"},"turnComplete":true}}"#;
        let msg: ServerMessage = serde_json::from_str(raw).unwrap();
        let sc = msg.server_content.unwrap();
        assert_eq!(sc.output_transcription.unwrap().text, "Hello");
        assert_eq!(sc.turn_complete, Some(true));
    }
}
