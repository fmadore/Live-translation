use super::spool::TimedChunk;

const RATE: usize = 16_000;
const WINDOW: usize = 10 * RATE;
const OVERLAP: usize = RATE;
const SILENCE: usize = RATE * 8 / 10;

pub struct Window {
    pub samples: Vec<f32>,
    pub start_ms: u64,
    pub consumed: usize,
    pub speech: bool,
}

#[derive(Default)]
pub struct Segmenter {
    samples: Vec<f32>,
    start_ms: u64,
    silent: usize,
    speech: bool,
}

impl Segmenter {
    pub fn push(&mut self, chunk: TimedChunk) -> Vec<Window> {
        let mut ready = Vec::new();
        let expected = self.start_ms + self.samples.len() as u64 / 16;
        // A pause/device gap must not collapse the transcript's time axis.
        if !self.samples.is_empty() && chunk.start_ms > expected + 250 {
            if let Some(window) = self.finish() {
                ready.push(window);
            }
        }
        if self.samples.is_empty() {
            self.start_ms = chunk.start_ms;
        }
        let samples: Vec<f32> = chunk
            .pcm
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
            .collect();
        // Conservative silence gate (~-80 dBFS). Quiet speech still reaches Whisper's own
        // no-speech classifier. Endpointing is an optimization, not an audio-loss policy.
        let active = samples.iter().any(|v| v.abs() > 0.0001);
        self.silent = if active {
            0
        } else {
            self.silent + samples.len()
        };
        self.speech |= active;
        self.samples.extend(samples);
        if self.silent >= SILENCE && self.samples.len() >= RATE {
            if let Some(window) = self.finish() {
                ready.push(window);
            }
        } else if self.samples.len() >= WINDOW {
            ready.push(self.take(OVERLAP));
        }
        ready
    }

    fn take(&mut self, keep: usize) -> Window {
        let samples = std::mem::take(&mut self.samples);
        let consumed = samples.len() - keep;
        self.samples = samples[consumed..].to_vec();
        let start_ms = self.start_ms;
        self.start_ms += consumed as u64 / 16;
        let speech = self.speech;
        self.speech = self.samples.iter().any(|v| v.abs() > 0.0001);
        self.silent = 0;
        Window {
            samples,
            start_ms,
            consumed,
            speech,
        }
    }

    pub fn finish(&mut self) -> Option<Window> {
        if self.samples.is_empty() {
            None
        } else {
            Some(self.take(0))
        }
    }
}

/// Only reconcile an actual time overlap; a repeated phrase later in a meeting is valid.
pub fn trim_overlap<'a>(previous: &str, current: &'a str, overlaps: bool) -> &'a str {
    if !overlaps {
        return current;
    }
    for (index, _) in current.char_indices().rev() {
        let prefix = &current[..index];
        if prefix.chars().count() >= 3 && previous.ends_with(prefix) {
            return current[index..].trim_start();
        }
    }
    if current.chars().count() >= 3 && previous.ends_with(current) {
        ""
    } else {
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(start_ms: u64, sample: i16) -> TimedChunk {
        TimedChunk {
            start_ms,
            pcm: sample.to_le_bytes().repeat(1600),
        }
    }
    #[test]
    fn windows_preserve_every_sample_including_short_final_tail() {
        let mut segmenter = Segmenter::default();
        let mut consumed = 0;
        let mut starts = Vec::new();
        for i in 0..205 {
            for window in segmenter.push(frame(i * 100, 100)) {
                consumed += window.consumed;
                starts.push(window.start_ms);
            }
        }
        consumed += segmenter.finish().unwrap().consumed;
        assert_eq!(consumed, 205 * 1600);
        assert_eq!(starts, vec![0, 9000]);
    }
    #[test]
    fn pause_gap_keeps_elapsed_timestamps_and_silence_does_not_infer() {
        let mut segmenter = Segmenter::default();
        segmenter.push(frame(1000, 100));
        let windows = segmenter.push(frame(5000, 0));
        assert_eq!(windows[0].start_ms, 1000);
        let tail = segmenter.finish().unwrap();
        assert_eq!(tail.start_ms, 5000);
        assert!(!tail.speech);
    }
    #[test]
    fn overlap_is_unicode_safe_and_does_not_remove_later_repetitions() {
        assert_eq!(
            trim_overlap("bonjour tout le monde", "tout le monde et bienvenue", true),
            "et bienvenue"
        );
        assert_eq!(trim_overlap("你好世界朋友", "世界朋友再见", true), "再见");
        assert_eq!(trim_overlap("yes yes", "yes yes", false), "yes yes");
    }
}
