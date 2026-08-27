//! Gemini Live integration: WebSocket clients and wire protocol. Two models share the one
//! endpoint — Live Translate for translated captions, Transcribe Live for subtitles.

pub mod client;
pub mod protocol;
pub mod transcribe;

pub use client::{GeminiConfig, DEFAULT_HOST, DEFAULT_TRANSLATE_MODEL};
pub use transcribe::{GeminiTranscribeConfig, DEFAULT_TRANSCRIBE_MODEL};
