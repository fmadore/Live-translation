//! System (loopback) audio capture — the "audio coming out of the laptop" path, e.g. a
//! remote speaker on a Zoom call. On Windows this uses WASAPI loopback, which captures the
//! render endpoint mix without any virtual audio device. On other platforms it is a stub.
//!
//! ⚠️ The Windows path below could not be compiled in the (Linux) scaffolding environment.
//! It follows the `wasapi` crate's loopback example; validate signatures against the
//! installed `wasapi` version and a real device during the rehearsal. Everything downstream
//! (resampling, chunking, leveling via `CaptureState`) is shared with the mic path and is
//! exercised by tests.

use anyhow::Result;
use tauri::AppHandle;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use super::AudioChunk;

#[cfg(not(windows))]
pub fn run_system_loopback(
    _app: AppHandle,
    _chunk_tx: UnboundedSender<AudioChunk>,
    _cancel: CancellationToken,
) -> Result<()> {
    anyhow::bail!("System (loopback) capture is only supported on Windows in this build")
}

#[cfg(windows)]
pub fn run_system_loopback(
    app: AppHandle,
    chunk_tx: UnboundedSender<AudioChunk>,
    cancel: CancellationToken,
) -> Result<()> {
    windows_impl::run(app, chunk_tx, cancel)
}

#[cfg(windows)]
mod windows_impl {
    use std::collections::VecDeque;

    use anyhow::{Context, Result};
    use tauri::AppHandle;
    use tokio::sync::mpsc::UnboundedSender;
    use tokio_util::sync::CancellationToken;
    use wasapi::{
        initialize_mta, Direction, SampleType, ShareMode, WaveFormat,
    };

    use crate::audio::capture::CaptureState;
    use crate::audio::AudioChunk;
    use crate::types::Origin;

    pub fn run(
        app: AppHandle,
        chunk_tx: UnboundedSender<AudioChunk>,
        cancel: CancellationToken,
    ) -> Result<()> {
        // COM must be initialised on the capture thread.
        initialize_mta().ok().context("failed to initialise COM (MTA)")?;

        let device = wasapi::get_default_device(&Direction::Render)
            .context("no default render device for loopback")?;
        let mut audio_client = device
            .get_iaudioclient()
            .context("failed to get IAudioClient")?;

        // Shared-mode mix format is what's actually playing; usually 32-bit float.
        let format: WaveFormat = audio_client
            .get_mixformat()
            .context("failed to get mix format")?;
        let in_rate = format.get_samplespersec();
        let channels = format.get_nchannels() as usize;
        let bits = format.get_bitspersample();
        let block_align = format.get_blockalign() as usize;
        let sample_type = format.get_subformat().unwrap_or(SampleType::Float);

        tracing::info!(
            in_rate,
            channels,
            bits,
            ?sample_type,
            "starting WASAPI loopback capture"
        );

        let (_def_period, min_period) = audio_client
            .get_periods()
            .context("failed to get device periods")?;

        // Loopback = render endpoint opened for capture.
        audio_client
            .initialize_client(
                &format,
                min_period,
                &Direction::Capture,
                &ShareMode::Shared,
                true,
            )
            .context("failed to initialise loopback client")?;

        let event = audio_client
            .set_get_eventhandle()
            .context("failed to set event handle")?;
        let capture_client = audio_client
            .get_audiocaptureclient()
            .context("failed to get capture client")?;

        audio_client
            .start_stream()
            .context("failed to start loopback stream")?;

        let mut state = CaptureState::new(Origin::System, in_rate, app, chunk_tx);
        let mut raw: VecDeque<u8> = VecDeque::new();
        let mut frame: Vec<f32> = Vec::new();

        while !cancel.is_cancelled() {
            // Drain whatever the device has buffered into `raw`.
            capture_client
                .read_from_device_to_deque(block_align, &mut raw)
                .context("failed to read loopback buffer")?;

            if !raw.is_empty() {
                frame.clear();
                decode_interleaved(&raw, sample_type, bits, &mut frame);
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
    fn decode_interleaved(bytes: &VecDeque<u8>, ty: SampleType, bits: u16, out: &mut Vec<f32>) {
        let buf: Vec<u8> = bytes.iter().copied().collect();
        match (ty, bits) {
            (SampleType::Float, 32) => {
                for c in buf.chunks_exact(4) {
                    out.push(f32::from_le_bytes([c[0], c[1], c[2], c[3]]));
                }
            }
            (SampleType::Int, 16) => {
                for c in buf.chunks_exact(2) {
                    out.push(i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0);
                }
            }
            (SampleType::Int, 32) => {
                for c in buf.chunks_exact(4) {
                    let v = i32::from_le_bytes([c[0], c[1], c[2], c[3]]);
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
