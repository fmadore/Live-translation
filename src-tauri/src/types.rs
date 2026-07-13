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
    /// BCP-47 code sent to the provider as the target language.
    pub fn bcp47(self) -> &'static str {
        match self {
            TargetLanguage::En => "en",
            TargetLanguage::Fr => "fr",
        }
    }
}

/// Translation provider / backend. Each has its own realtime API, audio rate, and key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    Gemini,
    OpenAi,
}

impl Provider {
    /// Sample rate the provider's realtime API expects on input, in Hz.
    pub fn input_sample_rate(self) -> u32 {
        match self {
            Provider::Gemini => 16_000,
            Provider::OpenAi => 24_000,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartOptions {
    pub source: AudioSource,
    pub target_language: TargetLanguage,
    /// Translation backend. Defaults to Gemini for older front-ends that omit it.
    #[serde(default)]
    pub provider: Provider,
    #[serde(default)]
    pub mic_device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Caption {
    pub turn_id: u64,
    pub text: String,
    pub source_text: String,
    /// `final` is a Rust keyword; serde exposes it to JS under the real name.
    #[serde(rename = "final")]
    pub final_: bool,
    pub origin: Origin,
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
    /// Which source this update is about; `None` means the whole session (e.g. stop).
    /// The operator UI aggregates per-origin states so concurrent sources don't clobber
    /// each other.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
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
