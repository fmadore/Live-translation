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
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use super::resample::{downmix_to_mono, f32_to_pcm16_le, LinearResampler};
use super::{AudioChunk, CHUNK_SAMPLES, TARGET_RATE};
use crate::types::{events, AudioDevice, AudioLevel, Origin};

/// Enumerate available input devices for the operator UI.
pub fn list_input_devices() -> Vec<AudioDevice> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());

    let mut out = Vec::new();
    if let Ok(devices) = host.input_devices() {
        for dev in devices {
            if let Ok(name) = dev.name() {
                let is_default = Some(&name) == default_name.as_ref();
                out.push(AudioDevice { name, is_default });
            }
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
            .find(|d| d.name().map(|n| n == wanted).unwrap_or(false))
            .ok_or_else(|| anyhow!("microphone '{}' not found", wanted)),
        None => host
            .default_input_device()
            .ok_or_else(|| anyhow!("no default microphone available")),
    }
}

/// Capture from the microphone until `cancel` fires. Blocks the calling thread.
pub fn run_microphone(
    app: AppHandle,
    device_name: Option<String>,
    chunk_tx: UnboundedSender<AudioChunk>,
    cancel: CancellationToken,
) -> Result<()> {
    let device = pick_device(device_name.as_deref())?;
    let config = device
        .default_input_config()
        .context("failed to read default input config")?;
    let sample_format = config.sample_format();
    let channels = config.channels() as usize;
    let in_rate = config.sample_rate().0;
    let stream_config: cpal::StreamConfig = config.into();

    tracing::info!(
        rate = in_rate,
        channels,
        ?sample_format,
        "starting microphone capture"
    );

    // Per-stream state captured by the callback.
    let mut state = CaptureState::new(Origin::Microphone, in_rate, app.clone(), chunk_tx);

    let err_app = app.clone();
    let err_fn = move |e: cpal::StreamError| {
        tracing::error!("microphone stream error: {e}");
        let _ = err_app.emit(events::STATUS, ());
    };

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| state.push_samples(data, channels),
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _| {
                let f: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                state.push_samples(&f, channels);
            },
            err_fn,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            &stream_config,
            move |data: &[u16], _| {
                let f: Vec<f32> = data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).collect();
                state.push_samples(&f, channels);
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
pub struct CaptureState {
    origin: Origin,
    app: AppHandle,
    chunk_tx: UnboundedSender<AudioChunk>,
    resampler: LinearResampler,
    mono_buf: Vec<f32>,
    resampled: Vec<f32>,
    // Accumulates resampled samples until we have a full 100 ms chunk.
    pending: Vec<f32>,
    last_level: Instant,
    peak_accum: f32,
    sq_sum: f64,
    sq_count: usize,
}

impl CaptureState {
    pub fn new(origin: Origin, in_rate: u32, app: AppHandle, chunk_tx: UnboundedSender<AudioChunk>) -> Self {
        Self {
            origin,
            app,
            chunk_tx,
            resampler: LinearResampler::new(in_rate, TARGET_RATE),
            mono_buf: Vec::with_capacity(4096),
            resampled: Vec::with_capacity(4096),
            pending: Vec::with_capacity(CHUNK_SAMPLES * 2),
            last_level: Instant::now(),
            peak_accum: 0.0,
            sq_sum: 0.0,
            sq_count: 0,
        }
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

        while self.pending.len() >= CHUNK_SAMPLES {
            let chunk: Vec<f32> = self.pending.drain(..CHUNK_SAMPLES).collect();
            let mut pcm = Vec::with_capacity(CHUNK_SAMPLES * 2);
            f32_to_pcm16_le(&chunk, &mut pcm);
            // If the consumer is gone the session is shutting down — stop quietly.
            if self.chunk_tx.send(AudioChunk { pcm_le: pcm }).is_err() {
                return;
            }
        }
    }

    fn maybe_emit_level(&mut self) {
        if self.last_level.elapsed() < Duration::from_millis(50) || self.sq_count == 0 {
            return;
        }
        let rms = (self.sq_sum / self.sq_count as f64).sqrt() as f32;
        let _ = self.app.emit(
            events::LEVEL,
            AudioLevel {
                source: self.origin,
                rms,
                peak: self.peak_accum,
            },
        );
        self.last_level = Instant::now();
        self.peak_accum = 0.0;
        self.sq_sum = 0.0;
        self.sq_count = 0;
    }
}
