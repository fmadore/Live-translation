//! Speech detection before Whisper, with Silero's voice-activity model (MIT), bundled with the
//! app (`resources/models/`).
//!
//! Each window is checked before it is decoded: one with no speech is not decoded at all, which
//! spares the processor and removes the text Whisper invents over room noise, music or a quiet
//! call; one with speech is decoded from just before its first words to just after its last.
//!
//! whisper.cpp's own `vad` switch is not used: it applies only in `whisper_full`, not in
//! `whisper_full_with_state`, which is what whisper-rs calls, so it would do nothing here.
//!
//! Gemini Transcribe uses the same model to find the pauses where it cuts the stream
//! (`gemini::cuts`), through [`Vad::load_for_cloud`] and [`Vad::probabilities`].

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::Manager;
use whisper_rs::{WhisperVadContext, WhisperVadContextParams, WhisperVadParams};

use crate::sync::lock;

/// The bundled model's place among the app's resources, and its exact size.
pub const RESOURCE: &str = "models/ggml-silero-v5.1.2.bin";
const BYTES: u64 = 885_098;

const SAMPLES_PER_CENTISECOND: f32 = 160.0;
/// Kept on each side of the speech found, so a soft first or last syllable is not cut.
const MARGIN: usize = 16_000 / 5;

pub struct Vad(Mutex<WhisperVadContext>);

/// The bundled model, where the installer put it.
pub fn bundled(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path()
        .resolve(RESOURCE, tauri::path::BaseDirectory::Resource)
        .ok()
}

impl Vad {
    /// The model for a cloud engine, which has not been through Whisper's processor check.
    /// whisper.cpp's speech detection runs on the same ggml library, built for a fixed
    /// instruction set (`cpu.rs`): on a processor that lacks it, the first window would be an
    /// illegal instruction. `None` (logged) there, as when the model will not load.
    pub fn load_for_cloud(path: &Path) -> Option<Self> {
        if let Err(missing) = super::cpu::ensure_supported() {
            tracing::info!("speech detection is unavailable on this processor ({missing})");
            return None;
        }
        Self::load(path)
    }

    /// The probability of speech in each 32 ms frame (512 samples) of `samples`, the last
    /// frame padded with silence; `None` on a failure. The model's memory starts afresh on
    /// each call, so a caller streaming audio passes some audio it has analysed already
    /// before the new.
    pub fn probabilities(&self, samples: &[f32]) -> Option<Vec<f32>> {
        let mut context = lock(&self.0);
        match context.detect_speech(samples) {
            Ok(()) => Some(context.probabilities().to_vec()),
            Err(error) => {
                tracing::warn!("speech detection failed: {error}");
                None
            }
        }
    }

    /// The model at `path`, or `None` (logged) when it is missing or will not load: Whisper then
    /// decodes every window, as it did before speech detection.
    pub fn load(path: &Path) -> Option<Self> {
        let usable = std::fs::metadata(path).is_ok_and(|m| m.len() == BYTES);
        let loaded = usable.then(|| path.to_str()).flatten().and_then(|path| {
            let mut params = WhisperVadContextParams::new();
            params.set_n_threads(2);
            params.set_use_gpu(false);
            WhisperVadContext::new(path, params).ok()
        });
        if loaded.is_none() {
            tracing::warn!(
                path = %path.display(),
                "speech detection is unavailable; Whisper will decode every window"
            );
        }
        loaded.map(|context| Self(Mutex::new(context)))
    }

    /// The part of `samples` that holds speech, with a margin; `None` when there is none. On a
    /// failure, all of it: speech detection saves work, it must never lose words.
    pub fn speech(&self, samples: &[f32]) -> Option<Range<usize>> {
        let found = lock(&self.0).segments_from_samples(WhisperVadParams::new(), samples);
        match found {
            Ok(segments) => span(segments.map(|s| (s.start, s.end)), samples.len()),
            Err(error) => {
                tracing::warn!("speech detection failed on a window: {error}");
                Some(0..samples.len())
            }
        }
    }
}

/// From the start of the first speech segment to the end of the last, in samples, widened by
/// [`MARGIN`] and kept within `len`. Segments are in centiseconds, as whisper.cpp gives them.
fn span(segments: impl Iterator<Item = (f32, f32)>, len: usize) -> Option<Range<usize>> {
    let (first, last) = segments.fold(None, |span: Option<(f32, f32)>, (start, end)| {
        Some(span.map_or((start, end), |(a, b)| (a.min(start), b.max(end))))
    })?;
    let start = to_samples(first).saturating_sub(MARGIN).min(len);
    let end = (to_samples(last) + MARGIN).min(len);
    (start < end).then_some(start..end)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to zero and rounded; a window is a few hundred thousand samples at most"
)]
fn to_samples(centiseconds: f32) -> usize {
    (centiseconds.max(0.0) * SAMPLES_PER_CENTISECOND).round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_span_runs_from_the_first_words_to_the_last_with_a_margin() {
        // Speech from 2.00 to 3.00 s and from 5.00 to 6.50 s in a 10 s window.
        let span = span([(200.0, 300.0), (500.0, 650.0)].into_iter(), 160_000).unwrap();
        assert_eq!(span, 32_000 - MARGIN..104_000 + MARGIN);
    }

    #[test]
    fn the_margin_stays_inside_the_window_and_silence_has_no_span() {
        assert_eq!(span([(0.0, 50.0)].into_iter(), 10_000), Some(0..10_000));
        assert_eq!(span(std::iter::empty(), 160_000), None);
        // A segment reported past the end of the window cannot reach beyond it.
        assert_eq!(
            span([(990.0, 1_200.0)].into_iter(), 160_000),
            Some(155_200..160_000)
        );
    }

    #[test]
    fn the_bundled_model_is_the_pinned_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(RESOURCE);
        assert_eq!(std::fs::metadata(&path).unwrap().len(), BYTES);
    }

    /// The real model on the rehearsal recording: speech is found, and silence is not.
    #[test]
    fn speech_is_found_in_the_rehearsal_and_not_in_silence() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
        let vad = Vad::load(&root.join(RESOURCE)).expect("the bundled model loads");
        let samples =
            crate::audio::fixture::load_fixture(&root.join("fixtures/rehearsal-en.wav")).unwrap();
        let found = vad.speech(&samples).expect("the rehearsal is speech");
        assert!(found.end - found.start > samples.len() / 2, "{found:?}");
        assert_eq!(vad.speech(&vec![0.0; 160_000]), None);
    }
}
