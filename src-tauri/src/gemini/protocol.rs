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
    /// Translate prompt for speech-to-text mode; absent for the dedicated translate model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    /// Empty object enables transcription of the recognised source speech.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: Option<Empty>,
    /// Empty object enables transcription of the translated output speech (translate mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_audio_transcription: Option<Empty>,
}

#[derive(Debug, Serialize)]
pub struct Empty {}

#[derive(Debug, Serialize)]
pub struct Content {
    pub parts: Vec<TextPart>,
}

#[derive(Debug, Serialize)]
pub struct TextPart {
    pub text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    pub response_modalities: Vec<String>,
    /// Only set for the dedicated speech-to-speech translate model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_config: Option<TranslationConfig>,
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
    /// Dedicated translate model, run in **TEXT** mode: it emits the translation as text only,
    /// with no audio synthesized — so there are no audio-output tokens to pay for. The translated
    /// text arrives via `outputTranscription`; `inputTranscription` carries the recognised source
    /// for the operator monitor. (Verified: this model accepts `responseModalities: ["TEXT"]` and
    /// still honours `translationConfig`.)
    pub fn live_translate(model: &str, target_language_code: &str) -> Self {
        SetupMessage {
            setup: Setup {
                model: format!("models/{model}"),
                generation_config: GenerationConfig {
                    response_modalities: vec!["TEXT".to_string()],
                    translation_config: Some(TranslationConfig {
                        target_language_code: target_language_code.to_string(),
                        echo_target_language: false,
                    }),
                },
                system_instruction: None,
                input_audio_transcription: Some(Empty {}),
                output_audio_transcription: None,
            },
        }
    }

    /// General Live model: audio in, translated TEXT out via a system instruction. No audio
    /// is synthesized. The translated text arrives in `serverContent.modelTurn.parts[].text`.
    pub fn speech_to_text(model: &str, target_language_name: &str) -> Self {
        let prompt = format!(
            "You are a live simultaneous interpreter at an academic conference. Translate the \
             incoming speech into {target_language_name}. Output only the {target_language_name} \
             translation — no commentary, labels, or quotation marks. If the speech is already in \
             {target_language_name}, repeat it verbatim. Preserve technical and academic terminology."
        );
        SetupMessage {
            setup: Setup {
                model: format!("models/{model}"),
                generation_config: GenerationConfig {
                    response_modalities: vec!["TEXT".to_string()],
                    translation_config: None,
                },
                system_instruction: Some(Content {
                    parts: vec![TextPart { text: prompt }],
                }),
                // Source transcription still drives the operator monitor.
                input_audio_transcription: Some(Empty {}),
                output_audio_transcription: None,
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
    /// In speech-to-text mode the translated text arrives here as TEXT parts. In translate
    /// mode this carries audio parts we ignore (the typed `text()` returns empty for those).
    #[serde(default)]
    pub model_turn: Option<ModelTurn>,
    #[serde(default)]
    pub turn_complete: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ModelTurn {
    #[serde(default)]
    pub parts: Vec<PartIn>,
}

impl ModelTurn {
    /// Concatenate the text of any text parts (ignores audio/inline-data parts).
    pub fn text(&self) -> String {
        self.parts
            .iter()
            .filter_map(|p| p.text.as_deref())
            .collect()
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct PartIn {
    #[serde(default)]
    pub text: Option<String>,
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
    fn live_translate_setup_serializes_to_expected_shape() {
        let v =
            serde_json::to_value(SetupMessage::live_translate("gemini-3.5-live-translate-preview", "en"))
                .unwrap();
        assert_eq!(
            v["setup"]["model"],
            "models/gemini-3.5-live-translate-preview"
        );
        assert_eq!(
            v["setup"]["generationConfig"]["responseModalities"][0],
            "TEXT"
        );
        assert_eq!(
            v["setup"]["generationConfig"]["translationConfig"]["targetLanguageCode"],
            "en"
        );
        assert!(v["setup"]["inputAudioTranscription"].is_object());
        // TEXT mode: no audio synthesized, so no output-audio transcription sidecar.
        assert!(v["setup"]["outputAudioTranscription"].is_null());
        assert!(v["setup"]["systemInstruction"].is_null());
    }

    #[test]
    fn speech_to_text_setup_uses_text_modality_and_prompt() {
        let v = serde_json::to_value(SetupMessage::speech_to_text("gemini-live-2.5-flash", "French"))
            .unwrap();
        assert_eq!(
            v["setup"]["generationConfig"]["responseModalities"][0],
            "TEXT"
        );
        // No translationConfig in this mode.
        assert!(v["setup"]["generationConfig"]["translationConfig"].is_null());
        assert!(v["setup"]["outputAudioTranscription"].is_null());
        let prompt = v["setup"]["systemInstruction"]["parts"][0]["text"]
            .as_str()
            .unwrap();
        assert!(prompt.contains("French"));
    }

    #[test]
    fn model_turn_text_concatenates_text_parts() {
        let raw = r#"{"serverContent":{"modelTurn":{"parts":[{"text":"Bon"},{"text":"jour"}]}}}"#;
        let msg: ServerMessage = serde_json::from_str(raw).unwrap();
        let mt = msg.server_content.unwrap().model_turn.unwrap();
        assert_eq!(mt.text(), "Bonjour");
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
