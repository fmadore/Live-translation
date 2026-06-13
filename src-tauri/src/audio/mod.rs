//! Audio capture: microphone (cpal, cross-platform) and system loopback (WASAPI, Windows).
//! Both feed a uniform stream of 16 kHz mono PCM-16 chunks to the Gemini client, and emit
//! a level meter to the UI.

pub mod capture;
pub mod loopback;
pub mod resample;

/// Sample rate Gemini Live expects on input.
pub const TARGET_RATE: u32 = 16_000;
/// We send ~100 ms chunks, the cadence recommended by the Live API.
pub const CHUNK_MS: usize = 100;
/// Samples per chunk at the target rate (16000 * 100 / 1000 = 1600).
pub const CHUNK_SAMPLES: usize = TARGET_RATE as usize * CHUNK_MS / 1000;

/// One 100 ms chunk of mono 16 kHz PCM, little-endian bytes, ready to base64-encode.
/// The source is implied by the channel it arrives on (one channel per origin).
#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub pcm_le: Vec<u8>,
}

pub use capture::list_input_devices;
