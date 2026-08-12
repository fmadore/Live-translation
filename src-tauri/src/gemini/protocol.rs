//! Wire types for Gemini 3.5 Live Translate's current WebSocket protocol.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct SetupMessage {
    pub setup: Setup,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Setup {
    pub model: String,
    pub generation_config: GenerationConfig,
    // Transcription sidecars are `BidiGenerateContentSetup` fields, not
    // `generationConfig` ones; the server rejects the whole setup if they are nested.
    pub input_audio_transcription: Empty,
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
pub struct RealtimeInput {
    pub audio: AudioBlob,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioBlob {
    pub mime_type: &'static str,
    pub data: String,
}

impl SetupMessage {
    pub fn live_translate(model: &str, target_language_code: &str) -> Self {
        Self {
            setup: Setup {
                model: format!("models/{model}"),
                generation_config: GenerationConfig {
                    // The current translate guide documents AUDIO output only. We discard
                    // audio payloads and use outputAudioTranscription for captions.
                    response_modalities: vec!["AUDIO".to_string()],
                    translation_config: TranslationConfig {
                        target_language_code: target_language_code.to_string(),
                        // Keep captions continuous in bilingual meetings: when speech is
                        // already in the target language, Gemini should transcribe it into
                        // the output sidecar instead of deliberately staying silent.
                        echo_target_language: true,
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
        Self {
            realtime_input: RealtimeInput {
                audio: AudioBlob {
                    mime_type: "audio/pcm;rate=16000",
                    data: base64_data,
                },
            },
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMessage {
    #[serde(default)]
    pub setup_complete: Option<serde_json::Value>,
    #[serde(default)]
    pub server_content: Option<ServerContent>,
    #[serde(default)]
    pub go_away: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerContent {
    #[serde(default)]
    pub input_transcription: Option<Transcription>,
    #[serde(default)]
    pub output_transcription: Option<Transcription>,
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
    fn live_translate_setup_matches_current_documented_shape() {
        let value = serde_json::to_value(SetupMessage::live_translate(
            "gemini-3.5-live-translate-preview",
            "en",
        ))
        .unwrap();
        let setup = &value["setup"];
        let generation = &setup["generationConfig"];
        assert_eq!(generation["responseModalities"][0], "AUDIO");
        assert_eq!(generation["translationConfig"]["targetLanguageCode"], "en");
        assert_eq!(generation["translationConfig"]["echoTargetLanguage"], true);

        // The transcription sidecars sit on setup itself. Nesting them under
        // generationConfig makes the server reject setup with "Cannot find field".
        assert!(setup["inputAudioTranscription"].is_object());
        assert!(setup["outputAudioTranscription"].is_object());
        assert!(generation.get("inputAudioTranscription").is_none());
        assert!(generation.get("outputAudioTranscription").is_none());
    }

    #[test]
    fn realtime_input_uses_current_audio_blob_shape() {
        let value = serde_json::to_value(RealtimeInputMessage::pcm16("AAA=".into())).unwrap();
        assert_eq!(
            value["realtimeInput"]["audio"]["mimeType"],
            "audio/pcm;rate=16000"
        );
        assert_eq!(value["realtimeInput"]["audio"]["data"], "AAA=");
    }

    #[test]
    fn parses_transcription_and_turn_complete() {
        let message: ServerMessage = serde_json::from_str(
            r#"{"serverContent":{"inputTranscription":{"text":"Hello"},"outputTranscription":{"text":"Bonjour"},"turnComplete":true}}"#,
        )
        .unwrap();
        let content = message.server_content.unwrap();
        assert_eq!(content.input_transcription.unwrap().text, "Hello");
        assert_eq!(content.output_transcription.unwrap().text, "Bonjour");
        assert_eq!(content.turn_complete, Some(true));
    }
}
