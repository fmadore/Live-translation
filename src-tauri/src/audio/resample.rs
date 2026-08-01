//! Streaming mono speech resampler. Linear interpolation preserves continuity across input
//! blocks; a four-stage low-pass filter suppresses frequencies above the new Nyquist limit
//! before downsampling so device-rate ultrasonic content does not alias into speech.

pub struct LinearResampler {
    /// Input samples consumed per output sample (`in_rate / out_rate`).
    step: f64,
    /// Fractional position within the current [prev, cur) input interval.
    frac: f64,
    prev: f32,
    has_prev: bool,
    passthrough: bool,
    lowpass_alpha: f32,
    lowpass_state: [f32; 4],
    has_filter_state: bool,
}

impl LinearResampler {
    pub fn new(in_rate: u32, out_rate: u32) -> Self {
        let downsampling = out_rate < in_rate;
        // One-pole coefficient at 90% of the output Nyquist frequency. Cascading four
        // stages gives materially stronger rejection without a large FIR or extra crate.
        let cutoff_hz = out_rate as f32 * 0.45;
        let lowpass_alpha = 1.0 - (-2.0 * std::f32::consts::PI * cutoff_hz / in_rate as f32).exp();
        Self {
            step: in_rate as f64 / out_rate as f64,
            frac: 0.0,
            prev: 0.0,
            has_prev: false,
            passthrough: in_rate == out_rate,
            lowpass_alpha: if downsampling { lowpass_alpha } else { 1.0 },
            lowpass_state: [0.0; 4],
            has_filter_state: false,
        }
    }

    /// Resample `input`, appending results to `out`.
    pub fn process(&mut self, input: &[f32], out: &mut Vec<f32>) {
        if self.passthrough {
            out.extend_from_slice(input);
            return;
        }
        for &input_sample in input {
            let cur = if self.lowpass_alpha < 1.0 {
                if !self.has_filter_state {
                    self.lowpass_state.fill(input_sample);
                    self.has_filter_state = true;
                } else {
                    let mut stage_input = input_sample;
                    for state in &mut self.lowpass_state {
                        *state += self.lowpass_alpha * (stage_input - *state);
                        stage_input = *state;
                    }
                }
                self.lowpass_state[3]
            } else {
                input_sample
            };
            if !self.has_prev {
                self.prev = cur;
                self.has_prev = true;
                continue;
            }
            while self.frac < 1.0 {
                let s = self.prev + (cur - self.prev) * self.frac as f32;
                out.push(s);
                self.frac += self.step;
            }
            self.frac -= 1.0;
            self.prev = cur;
        }
    }
}

/// Average interleaved frames down to a single mono channel.
pub fn downmix_to_mono(interleaved: &[f32], channels: usize, out: &mut Vec<f32>) {
    if channels <= 1 {
        out.extend_from_slice(interleaved);
        return;
    }
    for frame in interleaved.chunks_exact(channels) {
        let sum: f32 = frame.iter().copied().sum();
        out.push(sum / channels as f32);
    }
}

/// Convert f32 [-1, 1] samples to little-endian 16-bit PCM bytes.
pub fn f32_to_pcm16_le(samples: &[f32], out: &mut Vec<u8>) {
    out.reserve(samples.len() * 2);
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsample_48k_to_16k_thirds_the_rate() {
        let mut r = LinearResampler::new(48_000, 16_000);
        let input: Vec<f32> = (0..4800).map(|i| (i as f32 * 0.001).sin()).collect();
        let mut out = Vec::new();
        r.process(&input, &mut out);
        // ~1/3 of the input length, allowing for warm-up of one sample.
        let expected = input.len() / 3;
        assert!(
            (out.len() as i64 - expected as i64).abs() <= 2,
            "got {}",
            out.len()
        );
    }

    #[test]
    fn passthrough_when_rates_match() {
        let mut r = LinearResampler::new(16_000, 16_000);
        let input = vec![0.1, 0.2, 0.3];
        let mut out = Vec::new();
        r.process(&input, &mut out);
        assert_eq!(out, input);
    }

    #[test]
    fn downsampling_attenuates_content_above_output_nyquist() {
        let mut resampler = LinearResampler::new(48_000, 16_000);
        let input: Vec<f32> = (0..48_000)
            .map(|i| (2.0 * std::f32::consts::PI * 15_000.0 * i as f32 / 48_000.0).sin())
            .collect();
        let mut output = Vec::new();
        resampler.process(&input, &mut output);
        let rms = (output.iter().map(|s| s * s).sum::<f32>() / output.len() as f32).sqrt();
        assert!(rms < 0.2, "out-of-band RMS was {rms}");
    }

    #[test]
    fn pcm16_roundtrip_endianness() {
        let mut out = Vec::new();
        f32_to_pcm16_le(&[0.0, 1.0, -1.0], &mut out);
        assert_eq!(out, vec![0, 0, 0xFF, 0x7F, 0x01, 0x80]);
    }
}
