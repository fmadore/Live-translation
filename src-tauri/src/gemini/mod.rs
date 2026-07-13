//! Gemini Live integration: WebSocket client and wire protocol.

pub mod client;
pub mod protocol;

pub use client::{GeminiConfig, DEFAULT_HOST, DEFAULT_TRANSLATE_MODEL};
