//! Gemini Live integration: WebSocket client and wire protocol.

pub mod client;
pub mod protocol;

pub use client::{
    run_session, GeminiConfig, DEFAULT_HOST, DEFAULT_STT_MODEL, DEFAULT_TRANSLATE_MODEL,
};
