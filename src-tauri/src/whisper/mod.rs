//! Multilingual local transcription, or translation into English with Whisper's translate task.
//! Capture, disk ingestion and inference run independently; normal Stop closes input then
//! drains every accepted frame.
//!
//! Where the windows are decoded: on Windows, on the graphics processor through the Vulkan
//! helper (`gpu.rs`) when one is installed and finds a usable device; on a Mac, on the graphics
//! processor through Metal, in the app; otherwise, and whenever the graphics processor fails,
//! on the processor. Silero's speech detection (`vad.rs`) decides first whether a window is
//! worth decoding at all.
//!
//! `backend.rs` holds where windows are decoded, `pipeline.rs` how a source's audio becomes
//! captions.
mod backend;
pub mod cpu;
// Shared word for word with the Vulkan helper, so both decode a window the same way. Some of
// it (the helper's half of the messages) is only used there.
#[allow(dead_code)]
#[path = "../../whisper-vulkan/src/engine.rs"]
mod engine;
mod gpu;
mod language;
pub mod models;
mod pipeline;
mod prompt;
pub(crate) mod reconcile;
mod segment;
pub mod spool;
#[cfg(test)]
mod tests;
mod vad;

use anyhow::{Result, ensure};

// `Engine` is the `whisper-engine` event's payload, by this path in `types.rs`.
#[allow(unused_imports)]
pub use backend::Engine;
pub use backend::{LoadedModel, load};
pub use pipeline::{LocalSession, run};

pub fn validate_language(language: Option<&str>) -> Result<()> {
    let Some(language) = language else {
        return Ok(());
    };
    // These quantized multilingual tiny/base/small models have the original 99 languages.
    // Cantonese's separate token belongs to large-v3 and is deliberately not advertised.
    ensure!(!language.contains('\0'), "Invalid spoken language");
    ensure!(
        whisper_rs::get_lang_id(language).is_some_and(|id| id < 99),
        "Unsupported Whisper spoken language"
    );
    Ok(())
}
