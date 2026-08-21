//! Microphone capture via cpal. Cross-platform; this is the presenter-at-the-laptop path.
//!
//! The cpal stream callback runs on its own thread and is `!Send`, so `run_microphone`
//! is designed to be invoked inside a dedicated `std::thread`: it builds the stream,
//! starts it, then parks until cancelled, keeping the stream alive for its lifetime.

use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{error::TrySendError, Sender};
use tokio_util::sync::CancellationToken;

use super::resample::{downmix_to_mono, f32_to_pcm16_le, LinearResampler};
use super::{chunk_samples, AudioChunk};
use crate::types::{events, AudioDevice, AudioLevel, Origin, SessionState, StatusUpdate};

/// Enumerate available input devices for the operator UI.
pub fn list_input_devices() -> Vec<AudioDevice> {
    let host = cpal::default_host();
    let default_name = host.default_input_device().map(|device| device.to_string());

    let mut out = Vec::new();
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            let name = device.to_string();
            let is_default = Some(&name) == default_name.as_ref();
            out.push(AudioDevice { name, is_default });
        }
    }
    out
}

fn pick_device(name: Option<&str>) -> Result<cpal::Device> {
    let host = cpal::default_host();
    match name {
        Some(wanted) => host
            .input_devices()
            .context("failed to enumerate input devices")?
            .find(|device| device.to_string() == wanted)
            .ok_or_else(|| anyhow!("microphone '{}' not found", wanted)),
        None => host
            .default_input_device()
            .ok_or_else(|| anyhow!("no default microphone available")),
    }
}

/// Capture from the microphone until `cancel` fires. Blocks the calling thread.
///
/// Everything that can fail here fails *before* the stream starts — device resolution, config,
/// and the stream build below — and the errors stay technical on purpose. The operator-facing
/// wording, including the Windows privacy-setting guidance a packaged build needs, is added
/// where the failure becomes a `StatusUpdate`: `session::report_source_failure`.
pub fn run_microphone(
    app: AppHandle,
    device_name: Option<String>,
    target_rate: u32,
    level_tx: Sender<AudioLevel>,
    chunk_tx: Sender<AudioChunk>,
    cancel: CancellationToken,
) -> Result<()> {
    let device = pick_device(device_name.as_deref())?;
    let config = device
        .default_input_config()
        .context("failed to read default input config")?;
    let sample_format = config.sample_format();
    let channels = config.channels() as usize;
    let in_rate = config.sample_rate();
    let stream_config: cpal::StreamConfig = config.into();

    tracing::info!(
        rate = in_rate,
        channels,
        ?sample_format,
        "starting microphone capture"
    );

    // Per-stream state captured by the callback.
    let mut state = CaptureState::new(Origin::Microphone, in_rate, target_rate, level_tx, chunk_tx);

    let stream_error_cancel = cancel.clone();
    let err_fn = move |e: cpal::Error| {
        tracing::error!("microphone stream error: {e}");
        let _ = app.emit(
            events::STATUS,
            StatusUpdate {
                state: SessionState::Error,
                message: Some(format!("Microphone stream error: {e}")),
                origin: Some(Origin::Microphone),
            },
        );
        stream_error_cancel.cancel();
    };

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            stream_config,
            move |data: &[f32], _| state.push_samples(data, channels),
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            stream_config,
            move |data: &[i16], _| {
                state.push_converted(data.iter().map(|&s| s as f32 / 32768.0), channels)
            },
            err_fn,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            stream_config,
            move |data: &[u16], _| {
                state.push_converted(
                    data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0),
                    channels,
                )
            },
            err_fn,
            None,
        ),
        SampleFormat::F64 => device.build_input_stream(
            stream_config,
            move |data: &[f64], _| state.push_converted(data.iter().map(|&s| s as f32), channels),
            err_fn,
            None,
        ),
        SampleFormat::I8 => device.build_input_stream(
            stream_config,
            move |data: &[i8], _| {
                state.push_converted(data.iter().map(|&s| s as f32 / 128.0), channels)
            },
            err_fn,
            None,
        ),
        SampleFormat::I32 => device.build_input_stream(
            stream_config,
            move |data: &[i32], _| {
                state.push_converted(data.iter().map(|&s| s as f32 / 2_147_483_648.0), channels)
            },
            err_fn,
            None,
        ),
        SampleFormat::I64 => device.build_input_stream(
            stream_config,
            move |data: &[i64], _| {
                state.push_converted(
                    data.iter()
                        .map(|&s| s as f64 as f32 / 9_223_372_036_854_775_808.0_f32),
                    channels,
                )
            },
            err_fn,
            None,
        ),
        SampleFormat::U8 => device.build_input_stream(
            stream_config,
            move |data: &[u8], _| {
                state.push_converted(data.iter().map(|&s| (s as f32 - 128.0) / 128.0), channels)
            },
            err_fn,
            None,
        ),
        SampleFormat::U32 => device.build_input_stream(
            stream_config,
            move |data: &[u32], _| {
                state.push_converted(
                    data.iter()
                        .map(|&s| (s as f64 - 2_147_483_648.0) as f32 / 2_147_483_648.0),
                    channels,
                )
            },
            err_fn,
            None,
        ),
        SampleFormat::U64 => device.build_input_stream(
            stream_config,
            move |data: &[u64], _| {
                state.push_converted(
                    data.iter().map(|&s| {
                        ((s as f64 - 9_223_372_036_854_775_808.0) / 9_223_372_036_854_775_808.0)
                            as f32
                    }),
                    channels,
                )
            },
            err_fn,
            None,
        ),
        other => return Err(anyhow!("unsupported sample format: {other:?}")),
    }
    .context("failed to build input stream")?;

    stream.play().context("failed to start microphone stream")?;

    // Keep the stream alive until cancelled.
    while !cancel.is_cancelled() {
        std::thread::sleep(Duration::from_millis(100));
    }
    tracing::info!("microphone capture stopped");
    Ok(())
}

/// Mutable state shared into a cpal callback: resampling, chunk accumulation, level metering.
/// Scratch buffers are retained across callbacks; each completed PCM chunk owns the one
/// small allocation transferred to the async WebSocket pipeline.
pub struct CaptureState {
    origin: Origin,
    level_tx: Sender<AudioLevel>,
    chunk_tx: Sender<AudioChunk>,
    resampler: LinearResampler,
    // Samples in one ~100 ms chunk at the target rate.
    chunk_len: usize,
    conv_buf: Vec<f32>,
    mono_buf: Vec<f32>,
    resampled: Vec<f32>,
    // Accumulates resampled samples until we have a full ~100 ms chunk.
    pending: Vec<f32>,
    pending_start: usize,
    last_level: Instant,
    peak_accum: f32,
    sq_sum: f64,
    sq_count: usize,
}

impl CaptureState {
    pub fn new(
        origin: Origin,
        in_rate: u32,
        target_rate: u32,
        level_tx: Sender<AudioLevel>,
        chunk_tx: Sender<AudioChunk>,
    ) -> Self {
        let chunk_len = chunk_samples(target_rate);
        Self {
            origin,
            level_tx,
            chunk_tx,
            resampler: LinearResampler::new(in_rate, target_rate),
            chunk_len,
            conv_buf: Vec::with_capacity(4096),
            mono_buf: Vec::with_capacity(4096),
            resampled: Vec::with_capacity(4096),
            pending: Vec::with_capacity(chunk_len * 2),
            pending_start: 0,
            last_level: Instant::now(),
            peak_accum: 0.0,
            sq_sum: 0.0,
            sq_count: 0,
        }
    }

    /// Feed interleaved samples that first need converting to f32, without allocating:
    /// they go through a reused scratch buffer.
    pub fn push_converted(&mut self, samples: impl Iterator<Item = f32>, channels: usize) {
        let mut conv = std::mem::take(&mut self.conv_buf);
        conv.clear();
        conv.extend(samples);
        self.push_samples(&conv, channels);
        self.conv_buf = conv;
    }

    /// Feed interleaved f32 samples at the device rate.
    pub fn push_samples(&mut self, interleaved: &[f32], channels: usize) {
        self.mono_buf.clear();
        downmix_to_mono(interleaved, channels, &mut self.mono_buf);

        // Accumulate level stats over the mono signal (pre-resample is fine).
        for &s in &self.mono_buf {
            let a = s.abs();
            if a > self.peak_accum {
                self.peak_accum = a;
            }
            self.sq_sum += (s as f64) * (s as f64);
            self.sq_count += 1;
        }
        self.maybe_emit_level();

        self.resampled.clear();
        self.resampler.process(&self.mono_buf, &mut self.resampled);
        self.pending.extend_from_slice(&self.resampled);

        while self.pending.len() - self.pending_start >= self.chunk_len {
            let mut pcm = Vec::with_capacity(self.chunk_len * 2);
            let end = self.pending_start + self.chunk_len;
            f32_to_pcm16_le(&self.pending[self.pending_start..end], &mut pcm);
            self.pending_start = end;
            // Never block the real-time callback. A full queue means the caption client is
            // already behind, so bounded loss is preferable to unbounded caption latency.
            if let Err(error) = self.chunk_tx.try_send(AudioChunk { pcm_le: pcm }) {
                if matches!(error, TrySendError::Closed(_)) {
                    return;
                }
            }
        }

        // Compact occasionally instead of shifting the whole pending buffer every 100 ms.
        if self.pending_start >= self.chunk_len * 8
            || self.pending_start.saturating_mul(2) >= self.pending.len()
        {
            self.pending.copy_within(self.pending_start.., 0);
            self.pending
                .truncate(self.pending.len() - self.pending_start);
            self.pending_start = 0;
        }
    }

    fn maybe_emit_level(&mut self) {
        if self.last_level.elapsed() < Duration::from_millis(50) || self.sq_count == 0 {
            return;
        }
        let rms = (self.sq_sum / self.sq_count as f64).sqrt() as f32;
        // Sent through a channel: the webview IPC hop happens on the emitter task, not here
        // on the real-time audio thread.
        let _ = self.level_tx.try_send(AudioLevel {
            source: self.origin,
            rms,
            peak: self.peak_accum,
        });
        self.last_level = Instant::now();
        self.peak_accum = 0.0;
        self.sq_sum = 0.0;
        self.sq_count = 0;
    }
}
