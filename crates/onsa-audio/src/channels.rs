//! Mapping a source channel layout onto the output channel count.
//!
//! Sources are converted to the output channel count right after decoding, so
//! everything downstream (resampling, mixing, the output stage) works on one
//! layout.

/// Converts interleaved frames from `from` channels to `to` channels.
#[derive(Debug, Clone)]
pub struct ChannelMap {
    from: usize,
    to: usize,
    /// Row-major `to x from` mixing matrix.
    matrix: Vec<f32>,
}

/// Gain of a centre or surround channel folded into a front channel (-3 dB).
const FOLD: f32 = std::f32::consts::FRAC_1_SQRT_2;

impl ChannelMap {
    /// Builds the mapping. Both counts must be at least one.
    pub fn new(from: usize, to: usize) -> Self {
        let from = from.max(1);
        let to = to.max(1);
        let mut matrix = vec![0.0; to * from];
        let mut set = |out: usize, input: usize, gain: f32| matrix[out * from + input] = gain;

        if from == to {
            for channel in 0..from {
                set(channel, channel, 1.0);
            }
        } else if from == 1 {
            // Mono goes to the front pair, or to the only channel there is.
            for out in 0..to.min(2) {
                set(out, 0, 1.0);
            }
        } else if to == 1 {
            let gain = 1.0 / from as f32;
            for input in 0..from {
                set(0, input, gain);
            }
        } else if from < to {
            // Fewer source channels than outputs: keep them in place, leave
            // the extra outputs silent.
            for channel in 0..from {
                set(channel, channel, 1.0);
            }
        } else if to == 2 {
            // Down to stereo, assuming the usual WAV/FLAC order:
            // FL, FR, FC, LFE, BL, BR, SL, SR. The low frequency channel is
            // dropped, as ITU-R BS.775 does.
            let mut left = vec![0.0f32; from];
            let mut right = vec![0.0f32; from];
            left[0] = 1.0;
            right[1] = 1.0;
            if from >= 3 {
                left[2] = FOLD;
                right[2] = FOLD;
            }
            for (index, _) in (0..from).enumerate().skip(4) {
                if index % 2 == 0 {
                    left[index] = FOLD;
                } else {
                    right[index] = FOLD;
                }
            }
            // Normalise so a full-scale signal on every channel cannot clip.
            let norm = left.iter().sum::<f32>().max(right.iter().sum::<f32>());
            for input in 0..from {
                set(0, input, left[input] / norm);
                set(1, input, right[input] / norm);
            }
        } else {
            for channel in 0..to {
                set(channel, channel, 1.0);
            }
        }

        Self { from, to, matrix }
    }

    /// Source channel count.
    pub fn from(&self) -> usize {
        self.from
    }

    /// Output channel count.
    pub fn to(&self) -> usize {
        self.to
    }

    /// Whether the mapping leaves samples untouched.
    pub fn is_identity(&self) -> bool {
        self.from == self.to
    }

    /// Maps `input` (whole frames of `from` channels) and appends the result
    /// to `output`.
    pub fn map_into(&self, input: &[f32], output: &mut Vec<f32>) {
        if self.is_identity() {
            output.extend_from_slice(input);
            return;
        }
        for frame in input.chunks_exact(self.from) {
            for out in 0..self.to {
                let row = &self.matrix[out * self.from..(out + 1) * self.from];
                output.push(
                    row.iter()
                        .zip(frame)
                        .map(|(gain, sample)| gain * sample)
                        .sum(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(from: usize, to: usize, input: &[f32]) -> Vec<f32> {
        let mut out = Vec::new();
        ChannelMap::new(from, to).map_into(input, &mut out);
        out
    }

    #[test]
    fn identity_copies() {
        assert_eq!(map(2, 2, &[0.1, 0.2, 0.3, 0.4]), vec![0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn mono_fills_the_front_pair() {
        assert_eq!(map(1, 2, &[0.5, -0.5]), vec![0.5, 0.5, -0.5, -0.5]);
        assert_eq!(map(1, 4, &[0.5]), vec![0.5, 0.5, 0.0, 0.0]);
    }

    #[test]
    fn stereo_to_mono_averages() {
        assert_eq!(map(2, 1, &[1.0, 0.0]), vec![0.5]);
    }

    #[test]
    fn surround_to_stereo_keeps_the_centre_and_never_clips() {
        // 5.1 with only the centre channel lit.
        let centre = map(6, 2, &[0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
        assert!(centre[0] > 0.2 && (centre[0] - centre[1]).abs() < 1e-6);
        // Everything at full scale stays within range.
        let full = map(6, 2, &[1.0; 6]);
        assert!(
            full.iter().all(|sample| sample.abs() <= 1.0 + 1e-6),
            "{full:?}"
        );
    }
}
