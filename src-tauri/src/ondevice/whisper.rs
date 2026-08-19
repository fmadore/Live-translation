//! Local recognition with whisper.cpp, via `whisper-rs`.
//!
//! Whisper transcribes a *buffer*, not a stream, so realtime captions come from a sliding
//! window: audio accumulates into the current utterance, inference runs periodically to
//! produce an interim caption, and the utterance is committed when the speaker pauses. A
//! simple energy gate decides what counts as a pause, and also keeps the model away from
//! silence — whisper is prone to inventing text ("thank you", "sous-titrage…") when handed
//! nothing but room tone.
//!
//! Inference is expensive. Every knob below trades CPU against latency, and the defaults are
//! deliberately conservative because the app may be running *two* of these at once in
//! **Both** mode while also holding two live captures. They have not been tuned against real
//! conference audio on the event hardware; that needs a Windows machine and a rehearsal.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{Context, Result};
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

use super::{RecognitionEvent, Recognizer};

const SAMPLE_RATE: usize = 16_000;

/// Commit the utterance after this much trailing silence.
const SILENCE_FINALIZE_MS: usize = 800;

/// Re-run inference for an interim caption after this much new audio. Lower feels more live
/// and costs proportionally more CPU.
const PARTIAL_EVERY_MS: usize = 2_000;

/// Hard cap on one utterance. Whisper's receptive field is 30 s and quality degrades well
/// before that, so a speaker who never pauses still gets committed captions.
const MAX_UTTERANCE_MS: usize = 15_000;

/// Whisper needs roughly a second of audio to say anything useful; below this it mostly
/// hallucinates.
const MIN_INFERENCE_MS: usize = 1_000;

/// Mean-square threshold for "this chunk is silence". Compared against mean square rather
/// than RMS to avoid a square root per chunk.
const SILENCE_MEAN_SQUARE: f32 = 0.0001; // RMS 0.01

/// Leave headroom for two captures, two recognizers and the UI rather than taking the box.
const MAX_THREADS: usize = 4;

fn ms_to_samples(ms: usize) -> usize {
    SAMPLE_RATE * ms / 1000
}

/// Model weights are read-only and `WhisperContext` is `Send + Sync`, so **Both** mode shares
/// one copy rather than paying ~142 MB twice. Each origin still gets its own `WhisperState`,
/// which is where per-stream decode state lives.
fn shared_context(model_path: &Path) -> Result<Arc<WhisperContext>> {
    static MODELS: OnceLock<Mutex<HashMap<PathBuf, Arc<WhisperContext>>>> = OnceLock::new();
    let cache = MODELS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(context) = cache.get(model_path) {
        return Ok(Arc::clone(context));
    }

    tracing::info!(path = %model_path.display(), "loading whisper model");
    let context = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|error| anyhow::anyhow!("{error}"))
        .with_context(|| {
            format!(
                "failed to load the speech model at {}",
                model_path.display()
            )
        })?;
    let context = Arc::new(context);
    cache.insert(model_path.to_path_buf(), Arc::clone(&context));
    Ok(context)
}

pub struct WhisperRecognizer {
    state: WhisperState,
    /// Whisper language code (`en`, `fr`); `None` lets it auto-detect.
    language: Option<String>,
    threads: i32,
    /// Audio for the utterance in progress.
    window: Vec<f32>,
    /// Whether the window holds anything above the silence gate.
    has_speech: bool,
    /// Trailing silent samples, used to decide the utterance has ended.
    silence_run: usize,
    /// New samples since the last interim inference.
    since_partial: usize,
}

impl WhisperRecognizer {
    pub fn new(model_path: &Path, language: Option<&str>) -> Result<Self> {
        let context = shared_context(model_path)?;
        let state = context
            .create_state()
            .map_err(|error| anyhow::anyhow!("{error}"))
            .context("failed to create a whisper decode state")?;

        let threads = std::thread::available_parallelism()
            .map(|n| n.get().min(MAX_THREADS))
            .unwrap_or(2)
            .max(1) as i32;

        Ok(Self {
            state,
            language: language.map(str::to_string),
            threads,
            window: Vec::with_capacity(ms_to_samples(MAX_UTTERANCE_MS)),
            has_speech: false,
            silence_run: 0,
            since_partial: 0,
        })
    }

    fn reset_window(&mut self) {
        self.window.clear();
        self.has_speech = false;
        self.silence_run = 0;
        self.since_partial = 0;
    }

    /// Run inference over the current window. `None` means the window produced nothing worth
    /// showing.
    fn transcribe(&mut self) -> Result<Option<String>> {
        if self.window.len() < ms_to_samples(MIN_INFERENCE_MS) {
            return Ok(None);
        }

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(self.threads);
        // Same-language subtitles only. Whisper *can* translate to English, but that would
        // quietly turn the keyless engine into a translator and contradict the mode.
        params.set_translate(false);
        // Each pass re-reads an overlapping window. Carrying decoder context across passes
        // is what sends whisper into repetition loops here.
        params.set_no_context(true);
        params.set_single_segment(true);
        params.set_no_timestamps(true);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        if let Some(language) = self.language.as_deref() {
            params.set_language(Some(language));
        }

        // Disjoint field borrows: `state` is taken mutably while `window` is read.
        self.state
            .full(params, &self.window)
            .map_err(|error| anyhow::anyhow!("{error}"))
            .context("whisper inference failed")?;

        let mut text = String::new();
        for index in 0..self.state.full_n_segments() {
            let Some(segment) = self.state.get_segment(index) else {
                continue;
            };
            if let Ok(part) = segment.to_str_lossy() {
                text.push_str(&part);
            }
        }

        Ok(clean(&text))
    }
}

/// Whisper pads its output with a leading space, and on near-silence emits bracketed
/// annotations such as `[BLANK_AUDIO]`, `(soft music)` or `[Musique]` rather than nothing.
/// Those are not speech and must never reach the overlay.
fn clean(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let is_annotation =
        |open: char, close: char| trimmed.starts_with(open) && trimmed.ends_with(close);
    if is_annotation('[', ']') || is_annotation('(', ')') || is_annotation('*', '*') {
        return None;
    }
    Some(trimmed.to_string())
}

/// Mean square of a chunk, used as the silence gate.
fn mean_square(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32
}

impl Recognizer for WhisperRecognizer {
    fn name(&self) -> &'static str {
        "whisper.cpp"
    }

    fn accept(&mut self, samples: &[i16]) -> Result<Vec<RecognitionEvent>> {
        let chunk: Vec<f32> = samples
            .iter()
            .map(|&sample| sample as f32 / i16::MAX as f32)
            .collect();
        let silent = mean_square(&chunk) < SILENCE_MEAN_SQUARE;

        if silent && !self.has_speech {
            // Nothing said yet. Keep only a short pre-roll so the first word of an utterance
            // is not clipped, and never grow the buffer through a long quiet stretch.
            let preroll = ms_to_samples(SILENCE_FINALIZE_MS);
            self.window.extend_from_slice(&chunk);
            if self.window.len() > preroll {
                let excess = self.window.len() - preroll;
                self.window.drain(..excess);
            }
            return Ok(Vec::new());
        }

        self.window.extend_from_slice(&chunk);
        self.since_partial += chunk.len();
        if silent {
            self.silence_run += chunk.len();
        } else {
            self.silence_run = 0;
            self.has_speech = true;
        }

        // The speaker paused, or the utterance ran long: commit it.
        let ended = self.silence_run >= ms_to_samples(SILENCE_FINALIZE_MS);
        let overlong = self.window.len() >= ms_to_samples(MAX_UTTERANCE_MS);
        if ended || overlong {
            let text = self.transcribe()?;
            self.reset_window();
            // An empty final still closes a turn the caller already opened with a partial.
            return Ok(vec![RecognitionEvent::Final(text.unwrap_or_default())]);
        }

        if self.since_partial >= ms_to_samples(PARTIAL_EVERY_MS) {
            self.since_partial = 0;
            if let Some(text) = self.transcribe()? {
                return Ok(vec![RecognitionEvent::Partial(text)]);
            }
        }

        Ok(Vec::new())
    }

    fn finish(&mut self) -> Result<Vec<RecognitionEvent>> {
        if !self.has_speech {
            return Ok(Vec::new());
        }
        let text = self.transcribe()?;
        self.reset_window();
        Ok(text.map(RecognitionEvent::Final).into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_whisper_padding() {
        assert_eq!(
            clean("  bonjour tout le monde "),
            Some("bonjour tout le monde".into())
        );
    }

    #[test]
    fn drops_non_speech_annotations() {
        // Whisper emits these on near-silence; they must never reach the overlay.
        assert_eq!(clean("[BLANK_AUDIO]"), None);
        assert_eq!(clean(" (soft music) "), None);
        assert_eq!(clean("[Musique]"), None);
        assert_eq!(clean("*applause*"), None);
        assert_eq!(clean("   "), None);
    }

    #[test]
    fn keeps_speech_containing_brackets() {
        // Only a wholly bracketed line is an annotation.
        assert_eq!(
            clean("the value [see slide 4] is rising"),
            Some("the value [see slide 4] is rising".into())
        );
    }

    #[test]
    fn silence_gate_separates_room_tone_from_speech() {
        let silence = vec![0.0005_f32; 1600];
        let speech = vec![0.2_f32; 1600];
        assert!(mean_square(&silence) < SILENCE_MEAN_SQUARE);
        assert!(mean_square(&speech) >= SILENCE_MEAN_SQUARE);
        assert_eq!(mean_square(&[]), 0.0);
    }

    #[test]
    fn sample_conversion_matches_the_pipeline_rate() {
        assert_eq!(ms_to_samples(1000), 16_000);
        assert_eq!(ms_to_samples(SILENCE_FINALIZE_MS), 12_800);
    }
}
