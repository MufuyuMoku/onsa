//! Look-ahead peak limiter (SPEC §4.3).
//!
//! The signal is delayed by the look-ahead. For each incoming frame the gain
//! it needs is computed; a sliding minimum over the look-ahead window and a
//! moving average over the same length turn that into a smooth gain that is
//! guaranteed to be at or below what every delayed frame needs, so the
//! ceiling is never crossed. Release is a one-pole rise back to unity.
//!
//! The delay line always runs, also when limiting is switched off, so turning
//! the limiter on or off never shifts the signal in time (which would click).

use super::MAX_CHANNELS;

/// Look-ahead in milliseconds.
pub const LOOKAHEAD_MS: f32 = 5.0;
/// Default ceiling in dBFS.
pub const DEFAULT_CEILING_DB: f32 = -0.1;
/// Default release in milliseconds.
pub const DEFAULT_RELEASE_MS: f32 = 80.0;

/// The delay the limiter (and so the whole callback chain) adds at
/// `sample_rate`, in frames.
pub fn latency_frames(sample_rate: u32) -> usize {
    ((sample_rate.max(1) as f32 * LOOKAHEAD_MS / 1000.0) as usize).max(1)
}

/// A look-ahead peak limiter. Buffers are allocated in [`Limiter::new`];
/// [`Limiter::process_frame`] is real-time safe.
pub struct Limiter {
    channels: usize,
    lookahead: usize,
    sample_rate: f32,
    enabled: bool,
    ceiling: f32,
    release_coef: f32,
    /// Delayed frames, `lookahead` frames of `channels` samples.
    delay: Vec<f32>,
    delay_pos: usize,
    /// Monotonic deque for the sliding minimum: values and their indices.
    min_values: Vec<f32>,
    min_index: Vec<u64>,
    min_head: usize,
    min_len: usize,
    /// Moving average of the sliding minimum.
    average: Vec<f32>,
    average_pos: usize,
    average_sum: f64,
    /// Frames seen so far.
    counter: u64,
    gain: f32,
}

impl Limiter {
    /// Builds a limiter for `channels` channels at `sample_rate`.
    pub fn new(sample_rate: u32, channels: usize) -> Self {
        let channels = channels.clamp(1, MAX_CHANNELS);
        let rate = sample_rate.max(1) as f32;
        let lookahead = latency_frames(sample_rate);
        let window = lookahead + 1;
        let mut limiter = Self {
            channels,
            lookahead,
            sample_rate: rate,
            enabled: true,
            ceiling: super::db_to_gain(DEFAULT_CEILING_DB),
            release_coef: 0.0,
            delay: vec![0.0; lookahead * channels],
            delay_pos: 0,
            min_values: vec![1.0; window + 1],
            min_index: vec![0; window + 1],
            min_head: 0,
            min_len: 0,
            average: vec![1.0; lookahead],
            average_pos: 0,
            average_sum: lookahead as f64,
            counter: 0,
            gain: 1.0,
        };
        limiter.set_release_ms(DEFAULT_RELEASE_MS);
        limiter
    }

    /// Delay the limiter adds, in frames.
    pub fn latency(&self) -> usize {
        self.lookahead
    }

    /// Switches limiting on or off. The delay stays either way.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Sets the release time.
    pub fn set_release_ms(&mut self, release_ms: f32) {
        let frames = (release_ms.max(1.0) / 1000.0 * self.sample_rate).max(1.0);
        self.release_coef = 1.0 - (-1.0 / frames).exp();
    }

    /// Sets the ceiling in dBFS.
    pub fn set_ceiling_db(&mut self, ceiling_db: f32) {
        self.ceiling = super::db_to_gain(ceiling_db.min(0.0));
    }

    /// Gain currently applied, 1.0 when not limiting.
    pub fn gain(&self) -> f32 {
        self.gain
    }

    /// Whether the delay line holds nothing audible (below -140 dBFS) and
    /// the gain is at rest. A filter tail rarely reaches exactly zero.
    pub fn is_quiet(&self) -> bool {
        self.gain >= 0.999_999 && self.delay.iter().all(|sample| sample.abs() < 1e-7)
    }

    /// Limits one frame in place (it comes out `latency()` frames later).
    /// Returns whether the gain is below unity. Real-time safe.
    #[inline]
    pub fn process_frame(&mut self, frame: &mut [f32]) -> bool {
        let channels = self.channels.min(frame.len());

        // Gain this incoming frame needs.
        let mut peak = 0.0f32;
        for sample in &frame[..channels] {
            peak = peak.max(sample.abs());
        }
        let needed = if self.enabled && peak > self.ceiling {
            self.ceiling / peak
        } else {
            1.0
        };

        // Sliding minimum over the last lookahead + 1 needs.
        let capacity = self.min_values.len();
        while self.min_len > 0 {
            let back = (self.min_head + self.min_len - 1) % capacity;
            if self.min_values[back] >= needed {
                self.min_len -= 1;
            } else {
                break;
            }
        }
        let slot = (self.min_head + self.min_len) % capacity;
        self.min_values[slot] = needed;
        self.min_index[slot] = self.counter;
        self.min_len += 1;
        let oldest = self.counter.saturating_sub(self.lookahead as u64);
        while self.min_index[self.min_head] < oldest {
            self.min_head = (self.min_head + 1) % capacity;
            self.min_len -= 1;
        }
        let minimum = self.min_values[self.min_head];
        self.counter += 1;

        // Moving average of the minimum: smooth, and never above what the
        // frame leaving the delay needs.
        self.average_sum += f64::from(minimum) - f64::from(self.average[self.average_pos]);
        self.average[self.average_pos] = minimum;
        self.average_pos = (self.average_pos + 1) % self.lookahead;
        let smooth = (self.average_sum / self.lookahead as f64) as f32;

        self.gain = if smooth < self.gain {
            smooth
        } else {
            self.gain + (smooth - self.gain) * self.release_coef
        };

        // Swap the frame with the delayed one and apply the gain.
        let start = self.delay_pos * self.channels;
        for (channel, sample) in frame[..channels].iter_mut().enumerate() {
            let delayed = self.delay[start + channel];
            self.delay[start + channel] = *sample;
            let mut out = delayed * self.gain;
            if self.enabled {
                out = out.clamp(-self.ceiling, self.ceiling);
            }
            *sample = out;
        }
        self.delay_pos = (self.delay_pos + 1) % self.lookahead;
        self.gain < 0.999_999
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(limiter: &mut Limiter, input: &[f32]) -> Vec<f32> {
        let mut out = input.to_vec();
        for frame in out.chunks_exact_mut(2) {
            limiter.process_frame(frame);
        }
        out
    }

    fn sine(frames: usize, amplitude: f32) -> Vec<f32> {
        (0..frames)
            .flat_map(|n| {
                let v = (std::f32::consts::TAU * 1000.0 * n as f32 / 48_000.0).sin() * amplitude;
                [v, v]
            })
            .collect()
    }

    #[test]
    fn never_crosses_the_ceiling() {
        let mut limiter = Limiter::new(48_000, 2);
        let out = run(&mut limiter, &sine(48_000, 4.0));
        let ceiling = super::super::db_to_gain(DEFAULT_CEILING_DB);
        let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(peak <= ceiling + 1e-6, "peak {peak} over {ceiling}");
        // And it does not flatten the signal to silence either.
        assert!(peak > ceiling * 0.9);
    }

    #[test]
    fn catches_a_lone_spike_in_silence() {
        let mut limiter = Limiter::new(48_000, 2);
        let mut input = vec![0.0f32; 2 * 4800];
        input[2 * 1000] = 3.0;
        input[2 * 1000 + 1] = -3.0;
        let out = run(&mut limiter, &input);
        let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(peak <= super::super::db_to_gain(DEFAULT_CEILING_DB) + 1e-6);
        // The spike is still there, only lower, one look-ahead later.
        assert!(out[2 * (1000 + limiter.latency())].abs() > 0.5);
    }

    #[test]
    fn is_transparent_below_the_ceiling() {
        let mut limiter = Limiter::new(48_000, 2);
        let input = sine(4800, 0.5);
        let out = run(&mut limiter, &input);
        let latency = limiter.latency() * 2;
        assert!(out[..latency].iter().all(|v| *v == 0.0));
        assert_eq!(&out[latency..], &input[..input.len() - latency]);
    }

    #[test]
    fn releases_after_a_burst() {
        let mut limiter = Limiter::new(48_000, 2);
        run(&mut limiter, &sine(4800, 4.0));
        assert!(limiter.gain() < 0.5);
        run(&mut limiter, &sine(48_000, 0.1));
        assert!(limiter.gain() > 0.999, "gain {}", limiter.gain());
    }

    #[test]
    fn switched_off_keeps_the_delay_and_the_level() {
        let mut limiter = Limiter::new(48_000, 2);
        limiter.set_enabled(false);
        let input = sine(4800, 2.0);
        let out = run(&mut limiter, &input);
        let latency = limiter.latency() * 2;
        assert_eq!(&out[latency..], &input[..input.len() - latency]);
    }
}
