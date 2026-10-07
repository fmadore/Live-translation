//! Gemini Live integration: WebSocket clients and wire protocol. Two models share the one
//! endpoint — Live Translate for translated captions, Transcribe Live for subtitles.

pub mod client;
pub mod protocol;
pub mod transcribe;

use anyhow::{Context, Result};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;

pub use client::{GeminiConfig, DEFAULT_HOST, DEFAULT_TRANSLATE_MODEL};
pub use transcribe::{GeminiTranscribeConfig, DEFAULT_TRANSCRIBE_MODEL};

/// The handshake both models share: one endpoint, authenticated by a `key` query parameter
/// rather than a header; the model is named in the setup message. `provider` names the client
/// in the error, which the operator reads as the detail of a reconnecting status.
fn gemini_request(provider: &str, host: &str, api_key: &str) -> Result<Request> {
    format!(
        "wss://{host}/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key={api_key}"
    )
    .into_client_request()
    .with_context(|| format!("failed to build {provider} request"))
}
