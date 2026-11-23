//! OpenAI's Realtime API: the *translations* session (WebSocket client and wire protocol) for
//! translated captions, and the *transcription* session for subtitles (`transcribe`).

pub mod client;
pub mod protocol;
mod transcribe;

#[cfg(test)]
mod language_probe;
#[cfg(test)]
mod live_probe;

pub use client::{
    DEFAULT_OPENAI_HOST, DEFAULT_OPENAI_TRANSCRIBE_MODEL, DEFAULT_OPENAI_TRANSLATE_MODEL,
    OpenAiConfig,
};
pub use transcribe::{DEFAULT_OPENAI_LIVE_TRANSCRIBE_MODEL, OpenAiTranscribeConfig};
