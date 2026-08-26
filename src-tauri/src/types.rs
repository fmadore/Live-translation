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

/// Caption backend. The cloud providers each have their own realtime API, audio rate and
/// key; `OnDevice` is the credential-free bundled product demonstration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    Gemini,
    OpenAi,
    Mistral,
    /// Bundled deterministic demonstration. Keyless, offline, same-language only.
    OnDevice,
}

impl Provider {
    /// Sample rate the backend expects on input, in Hz.
    pub fn input_sample_rate(self) -> u32 {
        match self {
            Provider::Gemini => 16_000,
            Provider::OpenAi => 24_000,
            Provider::Mistral => 16_000,
            Provider::OnDevice => 16_000,
        }
    }

    /// Whether a provider API key must be present before a session can start. The built-in
    /// demonstration is the one path that works with no credential — which is what keeps
    /// provider keys out of the app's *primary* functionality.
    pub fn requires_api_key(self) -> bool {
        !matches!(self, Provider::OnDevice)
    }

    /// Whether the backend can produce translated captions. The built-in demonstration is
    /// same-language only.
    pub fn can_translate(self) -> bool {
        matches!(self, Provider::Gemini | Provider::OpenAi)
    }
}

/// What the audience should see. Mistral currently powers transcription-only subtitles;
/// Gemini and OpenAI power translated captions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    #[default]
    Translate,
    Transcribe,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartOptions {
    pub source: AudioSource,
    pub target_language: TargetLanguage,
    /// Output behavior. Defaults to translation for older front-ends that omit it.
    #[serde(default)]
    pub mode: OutputMode,
    /// Realtime backend. Defaults to Gemini for older front-ends that omit it.
    #[serde(default)]
    pub provider: Provider,
    #[serde(default)]
    pub mic_device_name: Option<String>,
    /// Rehearsal mode: play the bundled speech fixture for this language through the whole
    /// pipeline instead of capturing audio, so the overlay, move mode and the export can be
    /// exercised with no microphone and no sound in the room. Absent — the normal case — runs
    /// a live session. `source` and `mic_device_name` are ignored while it is set.
    /// See gate 2 in `docs/microsoft-store.md`.
    #[serde(default)]
    pub rehearsal: Option<TargetLanguage>,
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

/// Level-only capture, started from the preflight so an operator can confirm the room
/// microphone or the loopback is actually producing sound *before* committing to a session.
/// It opens the same devices as a session, discards every sample, and never reaches a
/// provider, so it costs nothing and stores nothing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTestUpdate {
    pub active: bool,
    /// Present when the test stopped because a device failed, not because it was stopped.
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
    /// Deliberately separate from `STATUS`: a preflight audio test is not a session, and
    /// must never move the operator UI's session state machine.
    pub const AUDIO_TEST: &str = "audio-test";
    /// The operator tried to close the window while something was still unsaved or running,
    /// and the core held the window open for an answer. See `lifecycle::CloseGuard`.
    pub const CLOSE_REQUESTED: &str = "close-requested";
    /// A tray menu entry the front-end has to carry out, because it needs the session or
    /// transcript state the renderer owns. Payload is `tray::TrayCommand`.
    pub const TRAY_COMMAND: &str = "tray-command";
}
