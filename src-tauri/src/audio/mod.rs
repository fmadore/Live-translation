//! Audio capture: microphone (cpal, cross-platform) and system loopback (WASAPI on Windows, a
//! Core Audio process tap on macOS).
//! Both feed a uniform stream of mono PCM-16 chunks to the active translation client at the
//! provider's input rate (16 kHz for Gemini, 24 kHz for OpenAI), and emit a level meter to the
//! UI. The target rate is chosen per session and threaded into capture.
//!
//! `fixture` is a third source of the same stream, standing in for a capture device: it plays
//! a bundled recording so rehearsal mode exercises the pipeline with no audio hardware at all.

pub mod capture;
// The COM guard for the threads `devices` and `loopback` spawn to use WASAPI directly.
#[cfg(windows)]
mod com;
pub mod fixture;
pub mod loopback;
pub mod resample;
// Core Audio process taps: the system-audio source on macOS.
#[cfg(target_os = "macos")]
mod tap;

/// We send ~100 ms chunks, the cadence the realtime APIs recommend.
pub const CHUNK_MS: usize = 100;

/// Samples in one ~100 ms chunk at `rate` Hz (e.g. 1600 at 16 kHz, 2400 at 24 kHz).
pub fn chunk_samples(rate: u32) -> usize {
    rate as usize * CHUNK_MS / 1000
}

/// One ~100 ms chunk of mono PCM-16, little-endian bytes, ready to base64-encode.
/// The source is implied by the channel it arrives on (one channel per origin).
#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub pcm_le: Vec<u8>,
}

/// A capture that cannot start or go on for a reason the operator can act on directly, and
/// that the interface words in full (`session::capture::source_failure` gives each its own error id):
/// there is no technical text underneath worth showing, so none is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(any(windows, target_os = "macos")),
    allow(
        dead_code,
        reason = "only the Windows and macOS system captures can lose an application or output"
    )
)]
pub enum CaptureLost {
    /// The application being captured has quit, or restarted under another process.
    ApplicationClosed,
    /// The output whose sound is being captured was unplugged, disabled or removed.
    OutputDisconnected,
    /// No microphone is connected at all, so there is nothing to open.
    NoMicrophone,
}

impl std::fmt::Display for CaptureLost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ApplicationClosed => "selected application has closed; select it again",
            Self::OutputDisconnected => "selected output device disconnected or disabled",
            Self::NoMicrophone => "no microphone is connected",
        })
    }
}

impl std::error::Error for CaptureLost {}

pub use capture::list_input_devices;

pub mod applications;
pub mod devices;

pub mod sink;
