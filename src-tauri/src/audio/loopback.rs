//! System (loopback) audio capture — the "audio coming out of the laptop" path: a remote
//! speaker on Zoom or Teams, a browser tab, a media player. On Windows this uses WASAPI
//! loopback, which captures the render endpoint mix — every application's output, whatever
//! it is — without any virtual audio device. On other platforms it is a stub.
//!
//! The Windows path follows the current `wasapi` event-driven shared-stream API.
//! Everything downstream (resampling, chunking, leveling via `CaptureState`) is shared
//! with the mic path and is exercised by tests.

use anyhow::Result;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use super::AudioChunk;
use crate::types::AudioLevel;

#[cfg(not(windows))]
pub fn run_system_loopback(
    _target_rate: u32,
    _level_tx: Sender<AudioLevel>,
    _chunk_tx: Sender<AudioChunk>,
    _cancel: CancellationToken,
) -> Result<()> {
    anyhow::bail!("System (loopback) capture is only supported on Windows in this build")
}

#[cfg(windows)]
pub fn run_system_loopback(
    target_rate: u32,
    level_tx: Sender<AudioLevel>,
    chunk_tx: Sender<AudioChunk>,
    cancel: CancellationToken,
) -> Result<()> {
    windows_impl::run(target_rate, level_tx, chunk_tx, cancel)
}

#[cfg(windows)]
mod windows_impl {
    use std::collections::VecDeque;

    use anyhow::{Context, Result};
    use tokio::sync::mpsc::Sender;
    use tokio_util::sync::CancellationToken;
    use wasapi::{initialize_mta, DeviceEnumerator, Direction, SampleType, StreamMode, WaveFormat};

    use crate::audio::capture::CaptureState;
    use crate::audio::AudioChunk;
    use crate::types::{AudioLevel, Origin};

    /// `wasapi`'s fallible calls return `Box<dyn Error>`, which is neither `Send` nor
    /// `Sync` and so cannot flow into `anyhow` through `?` or `.context()`. This bridges
    /// them by stringifying the error behind a static context message.
    trait WasapiCtx<T> {
        fn ctx(self, msg: &'static str) -> Result<T>;
    }

    impl<T, E: std::fmt::Display> WasapiCtx<T> for std::result::Result<T, E> {
        fn ctx(self, msg: &'static str) -> Result<T> {
            self.map_err(|e| anyhow::anyhow!("{msg}: {e}"))
        }
    }

    pub fn run(
        target_rate: u32,
        level_tx: Sender<AudioLevel>,
        chunk_tx: Sender<AudioChunk>,
        cancel: CancellationToken,
    ) -> Result<()> {
        // COM must be initialised on the capture thread. `initialize_mta` returns an
        // `HRESULT`; `.ok()` turns it into a `windows::core::Result` that anyhow accepts.
        initialize_mta()
            .ok()
            .context("failed to initialise COM (MTA)")?;

        let enumerator = DeviceEnumerator::new().ctx("failed to create device enumerator")?;
        let device = enumerator
            .get_default_device(&Direction::Render)
            .ctx("no default render device for loopback")?;
        let mut audio_client = device
            .get_iaudioclient()
            .ctx("failed to get IAudioClient")?;

        // Shared-mode mix format is what's actually playing; usually 32-bit float.
        let format: WaveFormat = audio_client
            .get_mixformat()
            .ctx("failed to get mix format")?;
        let in_rate = format.get_samplespersec();
        let channels = format.get_nchannels() as usize;
        let bits = format.get_bitspersample();
        let sample_type = format.get_subformat().unwrap_or(SampleType::Float);

        tracing::info!(
            in_rate,
            channels,
            bits,
            ?sample_type,
            "starting WASAPI loopback capture"
        );

        let (_default_period, min_period) = audio_client
            .get_device_period()
            .ctx("failed to get device periods")?;

        // Loopback = render endpoint opened for capture.
        let mode = StreamMode::EventsShared {
            autoconvert: true,
            buffer_duration_hns: min_period,
        };
        audio_client
            .initialize_client(&format, &Direction::Capture, &mode)
            .ctx("failed to initialise loopback client")?;

        let event = audio_client
            .set_get_eventhandle()
            .ctx("failed to set event handle")?;
        let capture_client = audio_client
            .get_audiocaptureclient()
            .ctx("failed to get capture client")?;

        audio_client
            .start_stream()
            .ctx("failed to start loopback stream")?;

        let mut state = CaptureState::new(Origin::System, in_rate, target_rate, level_tx, chunk_tx);
        let mut raw: VecDeque<u8> = VecDeque::new();
        let mut frame: Vec<f32> = Vec::new();

        while !cancel.is_cancelled() {
            // Drain whatever the device has buffered into `raw`.
            capture_client
                .read_from_device_to_deque(&mut raw)
                .ctx("failed to read loopback buffer")?;

            if !raw.is_empty() {
                frame.clear();
                decode_interleaved(&mut raw, sample_type, bits, &mut frame);
                raw.clear();
                state.push_samples(&frame, channels);
            }

            // Wake on the audio event; short timeout keeps cancellation responsive.
            if event.wait_for_event(100).is_err() {
                // Timeout — loop and re-check cancellation.
            }
        }

        let _ = audio_client.stop_stream();
        tracing::info!("WASAPI loopback capture stopped");
        Ok(())
    }

    /// Decode raw interleaved endpoint bytes into f32 samples in [-1, 1].
    /// `make_contiguous` rotates the deque in place instead of copying it out.
    fn decode_interleaved(bytes: &mut VecDeque<u8>, ty: SampleType, bits: u16, out: &mut Vec<f32>) {
        let buf: &[u8] = bytes.make_contiguous();
        match (ty, bits) {
            (SampleType::Float, 32) => {
                for c in buf.as_chunks::<4>().0 {
                    out.push(f32::from_le_bytes(*c));
                }
            }
            (SampleType::Int, 16) => {
                for c in buf.as_chunks::<2>().0 {
                    out.push(i16::from_le_bytes(*c) as f32 / 32768.0);
                }
            }
            (SampleType::Int, 32) => {
                for c in buf.as_chunks::<4>().0 {
                    let v = i32::from_le_bytes(*c);
                    out.push(v as f32 / 2_147_483_648.0);
                }
            }
            (_, b) => {
                // Unexpected format: emit silence of the right length rather than panic.
                let bytes_per = (b / 8).max(1) as usize;
                for _ in 0..(buf.len() / bytes_per) {
                    out.push(0.0);
                }
                tracing::warn!("unsupported loopback sample format: {ty:?}/{b}-bit");
            }
        }
    }
}
