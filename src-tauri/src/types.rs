//! Types shared with the front-end. These mirror `src/lib/types.ts` — keep them in sync.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioSource {
    Microphone,
    System,
    Both,
}

impl AudioSource {
    pub fn wants_mic(self) -> bool {
        matches!(self, AudioSource::Microphone | AudioSource::Both)
    }
    pub fn wants_system(self) -> bool {
        matches!(self, AudioSource::System | AudioSource::Both)
    }
}

/// A single capture source, used as the `origin` on captions/levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    Microphone,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetLanguage {
    En,
    Fr,
}

impl TargetLanguage {
    /// BCP-47 code sent to Gemini as `targetLanguageCode`.
    pub fn bcp47(self) -> &'static str {
        match self {
            TargetLanguage::En => "en",
            TargetLanguage::Fr => "fr",
        }
    }

    /// Human-readable name, used in the speech-to-text translate prompt.
    pub fn name(self) -> &'static str {
        match self {
            TargetLanguage::En => "English",
            TargetLanguage::Fr => "French",
        }
    }
}

/// How we get from speech to translated text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TranslationMode {
    /// Dedicated speech-to-speech translate model; captions come from the output
    /// transcription sidecar (audio is discarded).
    LiveTranslate,
    /// General Live model with TEXT output and a translate system instruction: audio in,
    /// translated text out, no audio synthesized.
    SpeechToText,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartOptions {
    pub source: AudioSource,
    pub target_language: TargetLanguage,
    pub mode: TranslationMode,
    #[serde(default)]
    pub mic_device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Caption {
    pub turn_id: u64,
    pub text: String,
    pub source_text: String,
    pub final_: bool,
    pub origin: Origin,
}

// `final` is a Rust keyword; expose it as `final` to JS.
impl Caption {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "turnId": self.turn_id,
            "text": self.text,
            "sourceText": self.source_text,
            "final": self.final_,
            "origin": self.origin,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioLevel {
    pub source: Origin,
    pub rms: f32,
    pub peak: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Idle,
    Connecting,
    Running,
    Reconnecting,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusUpdate {
    pub state: SessionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

/// Event names emitted to the front-end. Mirror `EVT` in types.ts.
pub mod events {
    pub const CAPTION: &str = "caption";
    pub const LEVEL: &str = "audio-level";
    pub const STATUS: &str = "status";
}
