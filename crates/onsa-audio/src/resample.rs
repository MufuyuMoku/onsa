//! Streaming sample rate conversion on top of rubato (SPEC §3.4).
//!
//! The wrapper hides rubato's chunking: frames go in in any amount, and the
//! frames that come out are already free of the resampler's start-up delay.
//! When the input ends, [`StreamResampler::finish`] flushes the tail so the
//! total output is exactly `round(input * ratio)` frames. That exactness is
//! what keeps the playhead and the gapless joins honest.

use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{
    Async, Fft, FixedAsync, FixedSync, Indexing, PolynomialDegree, Resampler,
    SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

use crate::error::{Error, Result};

/// Frames per resampler chunk on the fixed (input) side.
const CHUNK_FRAMES: usize = 1024;

/// Resampler quality preset (SPEC §3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResamplerQuality {
    /// Polynomial interpolation: cheapest, a little aliasing.
    Fast,
    /// Synchronous FFT resampler: fast and high quality for a fixed ratio.
    #[default]
    Balanced,
    /// Long sinc filter: the highest quality, the most CPU.
    Best,
}

/// A resampler fed and drained in arbitrary amounts.
pub struct StreamResampler {
    inner: Box<dyn Resampler<f32>>,
    from: u32,
    to: u32,
    channels: usize,
    /// Input collected for the next chunk.
    input: Vec<f32>,
    input_frames: usize,
    /// Scratch output of one chunk.
    output: Vec<f32>,
    /// Output frames still to drop to cancel the resampler delay.
    delay_left: usize,
    fed: u64,
    produced: u64,
    finished: bool,
}

impl StreamResampler {
    /// Builds a resampler from `from` Hz to `to` Hz for `channels` channels.
    pub fn new(from: u32, to: u32, channels: usize, quality: ResamplerQuality) -> Result<Self> {
        let failed = |reason: String| Error::Resampler { from, to, reason };
        let ratio = f64::from(to) / f64::from(from);
        let inner: Box<dyn Resampler<f32>> = match quality {
            ResamplerQuality::Fast => Box::new(
                Async::<f32>::new_poly(
                    ratio,
                    1.1,
                    PolynomialDegree::Cubic,
                    CHUNK_FRAMES,
                    channels,
                    FixedAsync::Input,
                )
                .map_err(|error| failed(error.to_string()))?,
            ),
            ResamplerQuality::Balanced => Box::new(
                Fft::<f32>::new(
                    from as usize,
                    to as usize,
                    CHUNK_FRAMES,
                    channels,
                    FixedSync::Input,
                )
                .map_err(|error| failed(error.to_string()))?,
            ),
            ResamplerQuality::Best => {
                let parameters = SincInterpolationParameters {
                    sinc_len: 256,
                    f_cutoff: None,
                    oversampling_factor: 256,
                    interpolation: SincInterpolationType::Cubic,
                    window: WindowFunction::BlackmanHarris2,
                };
                Box::new(
                    Async::<f32>::new_sinc(
                        ratio,
                        1.1,
                        &parameters,
                        CHUNK_FRAMES,
                        channels,
                        FixedAsync::Input,
                    )
                    .map_err(|error| failed(error.to_string()))?,
                )
            }
        };
        let delay_left = inner.output_delay();
        let input = vec![0.0; inner.input_frames_max() * channels];
        let output = vec![0.0; inner.output_frames_max() * channels];
        Ok(Self {
            inner,
            from,
            to,
            channels,
            input,
            input_frames: 0,
            output,
            delay_left,
            fed: 0,
            produced: 0,
            finished: false,
        })
    }

    /// Source sample rate.
    pub fn from_rate(&self) -> u32 {
        self.from
    }

    /// Output frames the input fed so far will turn into, in total.
    fn expected_output(&self) -> u64 {
        (self.fed as f64 * f64::from(self.to) / f64::from(self.from)).round() as u64
    }

    /// Feeds interleaved frames and appends whatever output they complete.
    pub fn feed(&mut self, mut samples: &[f32], out: &mut Vec<f32>) -> Result<()> {
        let channels = self.channels;
        while !samples.is_empty() {
            let needed = self.inner.input_frames_next() - self.input_frames;
            let frames = needed.min(samples.len() / channels);
            let start = self.input_frames * channels;
            self.input[start..start + frames * channels]
                .copy_from_slice(&samples[..frames * channels]);
            self.input_frames += frames;
            self.fed += frames as u64;
            samples = &samples[frames * channels..];
            if self.input_frames == self.inner.input_frames_next() {
                self.run_chunk(None, out)?;
            }
        }
        Ok(())
    }

    /// Flushes the tail once the input has ended. Afterwards the resampler
    /// has produced exactly `round(input * ratio)` frames in total.
    pub fn finish(&mut self, out: &mut Vec<f32>) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        self.finished = true;
        let expected = self.expected_output();
        let partial = self.input_frames;
        self.run_chunk(Some(partial), out)?;
        // The delay holds the last frames back; push silence until they are out.
        let mut guard = 0;
        while self.produced < expected && guard < 64 {
            self.run_chunk(Some(0), out)?;
            guard += 1;
        }
        let excess = self.produced.saturating_sub(expected) as usize;
        if excess > 0 {
            let keep = out.len().saturating_sub(excess * self.channels);
            out.truncate(keep);
            self.produced = expected;
        }
        Ok(())
    }

    fn run_chunk(&mut self, partial: Option<usize>, out: &mut Vec<f32>) -> Result<()> {
        let channels = self.channels;
        let in_frames = self.input.len() / channels;
        let out_frames = self.output.len() / channels;
        let failed = |reason: String| Error::Resampler {
            from: self.from,
            to: self.to,
            reason,
        };
        let input = InterleavedSlice::new(&self.input[..], channels, in_frames)
            .map_err(|error| failed(error.to_string()))?;
        let mut output = InterleavedSlice::new_mut(&mut self.output[..], channels, out_frames)
            .map_err(|error| failed(error.to_string()))?;
        let mut indexing = Indexing::new();
        indexing.partial_len = partial;
        let (_, produced) = self
            .inner
            .process_into_buffer(&input, &mut output, Some(&indexing))
            .map_err(|error| Error::Resampler {
                from: self.from,
                to: self.to,
                reason: error.to_string(),
            })?;
        self.input_frames = 0;

        let skip = self.delay_left.min(produced);
        self.delay_left -= skip;
        let usable = produced - skip;
        out.extend_from_slice(&self.output[skip * channels..(skip + usable) * channels]);
        self.produced += usable as u64;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frames: usize, rate: u32, freq: f32) -> Vec<f32> {
        (0..frames)
            .flat_map(|n| {
                let v = (n as f32 * freq * std::f32::consts::TAU / rate as f32).sin() * 0.5;
                [v, v]
            })
            .collect()
    }

    #[test]
    fn output_length_is_exact_for_every_preset() {
        for quality in [
            ResamplerQuality::Fast,
            ResamplerQuality::Balanced,
            ResamplerQuality::Best,
        ] {
            let input = sine(44_100 + 123, 44_100, 440.0);
            let mut resampler = StreamResampler::new(44_100, 48_000, 2, quality).unwrap();
            let mut out = Vec::new();
            // Uneven feeding must not matter.
            for piece in input.chunks(2 * 777) {
                resampler.feed(piece, &mut out).unwrap();
            }
            resampler.finish(&mut out).unwrap();
            let expected = ((44_100.0 + 123.0) * 48_000.0 / 44_100.0_f64).round() as usize;
            assert_eq!(out.len() / 2, expected, "{quality:?}");
        }
    }

    #[test]
    fn a_resampled_sine_stays_in_phase() {
        // After delay compensation, output frame k must sit at input time
        // k / ratio; compare against the ideal sine at the output rate.
        let input = sine(48_000, 44_100, 440.0);
        let mut resampler =
            StreamResampler::new(44_100, 48_000, 2, ResamplerQuality::Balanced).unwrap();
        let mut out = Vec::new();
        resampler.feed(&input, &mut out).unwrap();
        resampler.finish(&mut out).unwrap();
        let ideal = sine(out.len() / 2, 48_000, 440.0);
        let middle = out.len() / 4..out.len() / 2;
        let error = middle
            .map(|i| (out[i] - ideal[i]).abs())
            .fold(0.0f32, f32::max);
        assert!(error < 0.01, "phase error {error}");
    }
}
