//! The offline sink: renders the engine's output into memory instead of a
//! device. Every automated engine test goes through it (SPEC §3.1, §12).
//!
//! It runs exactly the same [`OutputStage`] as the device callback. The one
//! difference is timing: a device cannot wait, so an empty buffer becomes
//! silence; the offline sink waits for the engine instead, which keeps the
//! rendered signal free of scheduling accidents and the tests deterministic.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::stage::{OutputStage, Shared};

/// Frames rendered per step, about what a device callback asks for.
const STEP_FRAMES: usize = 256;

/// Longest the sink waits for the engine before it renders silence anyway.
const WAIT_LIMIT: Duration = Duration::from_secs(10);

/// Renders the engine's output into memory.
pub struct OfflineSink {
    stage: OutputStage,
    shared: Arc<Shared>,
    sample_rate: u32,
}

impl OfflineSink {
    pub(crate) fn new(stage: OutputStage, shared: Arc<Shared>, sample_rate: u32) -> Self {
        Self {
            stage,
            shared,
            sample_rate,
        }
    }

    /// Output sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Output channel count.
    pub fn channels(&self) -> usize {
        self.stage.channels()
    }

    /// Whether the engine has nothing more to play and everything has been
    /// rendered.
    pub fn is_idle(&self) -> bool {
        !self.shared.producing.load(Ordering::Acquire)
            && !self.shared.flush_pending()
            && self.stage.buffered_frames() == 0
    }

    /// Waits until the engine has buffered at least `frames` frames, or has
    /// stopped producing. Returns whether the audio arrived.
    pub fn wait_for_audio(&self, frames: usize) -> bool {
        let deadline = Instant::now() + WAIT_LIMIT;
        while Instant::now() < deadline {
            if self.stage.buffered_frames() >= frames {
                return true;
            }
            std::thread::sleep(Duration::from_micros(200));
        }
        false
    }

    /// Renders `frames` frames of interleaved output.
    pub fn render(&mut self, frames: usize) -> Vec<f32> {
        let channels = self.channels();
        let mut out = vec![0.0; frames * channels];
        let mut done = 0;
        while done < frames {
            let step = STEP_FRAMES.min(frames - done);
            self.wait_for_step(step);
            self.stage
                .process(&mut out[done * channels..(done + step) * channels]);
            done += step;
        }
        out
    }

    /// Renders until the engine is idle, or `max_frames` frames at most.
    pub fn render_to_end(&mut self, max_frames: usize) -> Vec<f32> {
        let channels = self.channels();
        let mut out = Vec::new();
        while out.len() / channels < max_frames {
            if self.is_idle() && self.shared.silent.load(Ordering::Acquire) {
                break;
            }
            let step = STEP_FRAMES.min(max_frames - out.len() / channels);
            self.wait_for_step(step);
            let start = out.len();
            out.resize(start + step * channels, 0.0);
            self.stage.process(&mut out[start..]);
        }
        out
    }

    /// Waits until the stage can fill a whole step with real audio, unless
    /// it is fading out, holding, or the engine has nothing more to give.
    fn wait_for_step(&self, frames: usize) {
        let deadline = Instant::now() + WAIT_LIMIT;
        loop {
            let holding = self.shared.flush_pending() || self.shared.paused.load(Ordering::Acquire);
            let producing = self.shared.producing.load(Ordering::Acquire);
            if holding || !producing || self.stage.buffered_frames() >= frames {
                return;
            }
            if Instant::now() >= deadline {
                return;
            }
            std::thread::sleep(Duration::from_micros(100));
        }
    }
}
