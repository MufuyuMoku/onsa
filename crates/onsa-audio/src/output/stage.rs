//! The output stage: the only code that runs inside the device callback.
//!
//! Real-time rules (CLAUDE.md, SPEC §3.1): no allocation, no lock, no I/O, no
//! logging and no panic in [`OutputStage::process`]. It talks to the engine
//! thread through the atomics in [`Shared`] and two lock-free `rtrb` queues
//! allocated up front: audio in, DSP settings in. A third one carries samples
//! out to the analysis thread, but only while analysis is on.
//!
//! The stage owns the micro-fade (SPEC §3.3): pausing ramps the gain down
//! before it stops reading, resuming ramps it back up, and a flush request
//! (seek, stop, output change) ramps down, discards what is buffered and then
//! waits for fresh audio, which it ramps up again. After the fade comes the
//! callback side of the DSP chain (SPEC §4): EQ, preamp, limiter, volume and
//! dither.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use rtrb::{Consumer, Producer, RingBuffer};

use crate::dsp::chain::{Chain, ChainParams};
use crate::fade::{micro_fade_gain, MICRO_FADE_SECONDS};

/// Settings updates that can wait in the queue at once.
const UPDATE_QUEUE: usize = 16;
/// Length of the analysis tap.
const TAP_SECONDS: f32 = 0.5;

/// State shared between the engine thread and the output stage.
#[derive(Debug, Default)]
pub struct Shared {
    /// Frames taken out of the ring buffer, heard or discarded.
    pub consumed: AtomicU64,
    /// Set by the engine: hold playback, after a fade-out.
    pub paused: AtomicBool,
    /// Set by the engine: more audio is on its way. While false, an empty
    /// ring buffer is expected silence rather than an underrun.
    pub producing: AtomicBool,
    /// Bumped by the engine to ask for a flush.
    pub flush_request: AtomicU32,
    /// Set by the stage to the request it has completed.
    pub flush_ack: AtomicU32,
    /// Blocks where the stage wanted audio and found none.
    pub underruns: AtomicU64,
    /// Whether nothing is audible any more: the fade is down and the DSP
    /// chain has nothing left ringing or delayed.
    pub silent: AtomicBool,
    /// Set by the engine: copy the output to the analysis tap.
    pub analysis: AtomicBool,
    /// Set by the stage when the limiter reduced the gain; cleared by
    /// whoever reports it.
    pub limiting: AtomicBool,
}

impl Shared {
    /// Whether a flush has been asked for and not yet completed.
    pub fn flush_pending(&self) -> bool {
        self.flush_request.load(Ordering::Acquire) != self.flush_ack.load(Ordering::Acquire)
    }
}

/// The stage's end of the DSP plumbing.
pub struct StageDsp {
    chain: Chain,
    updates: Consumer<ChainParams>,
    tap: Producer<f32>,
}

/// The engine's end of the DSP plumbing of one output.
pub struct DspPort {
    /// Where new settings go.
    pub updates: Producer<ChainParams>,
    /// Post-DSP samples, for the analysis thread (taken when it starts).
    pub tap: Option<Consumer<f32>>,
    /// Delay the chain adds, in frames.
    pub latency: usize,
}

/// Builds both ends of the DSP plumbing for an output. `integer_bits` is the
/// width of an integer output format, `None` for float.
pub fn dsp_plumbing(
    sample_rate: u32,
    channels: usize,
    integer_bits: Option<u32>,
    params: ChainParams,
) -> (StageDsp, DspPort) {
    let chain = Chain::new(sample_rate, channels, integer_bits, params);
    let latency = chain.latency();
    let (update_tx, update_rx) = RingBuffer::new(UPDATE_QUEUE);
    let tap_len = ((sample_rate as f32 * TAP_SECONDS) as usize).max(4096) * channels.max(1);
    let (tap_tx, tap_rx) = RingBuffer::new(tap_len);
    (
        StageDsp {
            chain,
            updates: update_rx,
            tap: tap_tx,
        },
        DspPort {
            updates: update_tx,
            tap: Some(tap_rx),
            latency,
        },
    )
}

/// Reads the ring buffer, applies the micro-fade envelope and the DSP chain.
pub struct OutputStage {
    consumer: Consumer<f32>,
    shared: Arc<Shared>,
    channels: usize,
    fade_frames: u32,
    /// Position on the fade ramp, `0..=fade_frames`.
    gain_step: u32,
    /// The last frame read, before gain; allocated here, reused forever.
    last: Vec<f32>,
    dsp: StageDsp,
}

impl OutputStage {
    /// Creates the stage for `channels` channels at `sample_rate`.
    pub fn new(
        consumer: Consumer<f32>,
        shared: Arc<Shared>,
        channels: usize,
        sample_rate: u32,
        dsp: StageDsp,
    ) -> Self {
        let fade_frames = ((sample_rate as f32 * MICRO_FADE_SECONDS) as u32).max(1);
        shared.silent.store(true, Ordering::Release);
        Self {
            consumer,
            shared,
            channels: channels.max(1),
            fade_frames,
            gain_step: 0,
            last: vec![0.0; channels.max(1)],
            dsp,
        }
    }

    /// Channel count of the interleaved output.
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Frames of audio currently waiting in the ring buffer.
    pub fn buffered_frames(&self) -> usize {
        self.consumer.slots() / self.channels
    }

    /// Fills `out` (interleaved) and returns how many frames came from the
    /// ring buffer. Real-time safe.
    pub fn process(&mut self, out: &mut [f32]) -> usize {
        // Newest settings first; older queued ones are superseded.
        let mut latest = None;
        while let Ok(params) = self.dsp.updates.pop() {
            latest = Some(params);
        }
        if let Some(params) = latest {
            self.dsp.chain.set_params(params);
        }

        let taken = self.read_with_fade(out);

        if self.dsp.chain.process(out) {
            self.shared.limiting.store(true, Ordering::Relaxed);
        }
        if self.shared.analysis.load(Ordering::Relaxed) {
            // Whole frames only, so channels never slip in the analysis.
            let room = self.dsp.tap.slots() / self.channels * self.channels;
            let count = room.min(out.len());
            let _ = self.dsp.tap.push_partial_slice(&out[..count]);
        }

        self.shared
            .consumed
            .fetch_add(taken as u64, Ordering::AcqRel);
        self.shared.silent.store(
            self.gain_step == 0 && self.dsp.chain.is_silent(),
            Ordering::Release,
        );
        taken
    }

    /// Reads audio into `out` with the micro-fade applied.
    fn read_with_fade(&mut self, out: &mut [f32]) -> usize {
        let channels = self.channels;
        let frames = out.len() / channels;
        let flush = self.shared.flush_pending();
        let hold = flush || self.shared.paused.load(Ordering::Acquire);

        let mut written = 0;
        let mut taken = 0;

        while written < frames {
            if hold && self.gain_step == 0 {
                // Fully faded out: nothing more is heard.
                if flush {
                    self.drain();
                }
                break;
            }

            let available = self.consumer.slots() / channels;
            if available == 0 {
                if self.shared.producing.load(Ordering::Acquire) && !hold {
                    self.shared.underruns.fetch_add(1, Ordering::Relaxed);
                }
                // Nothing left to read: let the last frame die away along the
                // fade ramp instead of dropping to zero, so a buffer that runs
                // dry (end of stream, underrun, a flush with little buffered)
                // never clicks. What plays next fades in from silence.
                while written < frames && self.gain_step > 0 {
                    self.gain_step -= 1;
                    let gain = micro_fade_gain(self.gain_step as f32 / self.fade_frames as f32);
                    let target = &mut out[written * channels..(written + 1) * channels];
                    for (sample, value) in target.iter_mut().zip(&self.last) {
                        *sample = value * gain;
                    }
                    written += 1;
                }
                if hold && self.gain_step == 0 && written < frames {
                    // Faded out: go round once more to finish a flush.
                    continue;
                }
                break;
            }

            let count = available.min(frames - written);
            let Ok(chunk) = self.consumer.read_chunk(count * channels) else {
                break;
            };
            let (first, second) = chunk.as_slices();
            let mut used = 0;
            for frame in first
                .chunks_exact(channels)
                .chain(second.chunks_exact(channels))
            {
                if hold && self.gain_step == 0 {
                    break;
                }
                if hold {
                    self.gain_step -= 1;
                } else if self.gain_step < self.fade_frames {
                    self.gain_step += 1;
                }
                self.last.copy_from_slice(frame);
                let gain = micro_fade_gain(self.gain_step as f32 / self.fade_frames as f32);
                let target = &mut out[(written + used) * channels..(written + used + 1) * channels];
                for (sample, value) in target.iter_mut().zip(frame) {
                    *sample = value * gain;
                }
                used += 1;
            }
            chunk.commit(used * channels);
            written += used;
            taken += used;
            if used < count {
                // The fade reached zero inside this chunk.
                continue;
            }
        }

        out[written * channels..].fill(0.0);
        taken
    }

    /// Discards everything buffered and completes the pending flush.
    fn drain(&mut self) {
        let request = self.shared.flush_request.load(Ordering::Acquire);
        let slots = self.consumer.slots();
        let whole = slots - slots % self.channels;
        if let Ok(chunk) = self.consumer.read_chunk(whole) {
            chunk.commit_all();
        }
        self.shared
            .consumed
            .fetch_add((whole / self.channels) as u64, Ordering::AcqRel);
        self.shared.flush_ack.store(request, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::chain::DspSettings;

    const RATE: u32 = 1000;

    fn stage(capacity_frames: usize) -> (rtrb::Producer<f32>, OutputStage, Arc<Shared>, DspPort) {
        let (producer, consumer) = rtrb::RingBuffer::new(capacity_frames * 2);
        let shared = Arc::new(Shared::default());
        shared.producing.store(true, Ordering::Release);
        let (dsp, port) = dsp_plumbing(RATE, 2, None, DspSettings::default().chain_params(RATE));
        let stage = OutputStage::new(consumer, shared.clone(), 2, RATE, dsp);
        (producer, stage, shared, port)
    }

    #[test]
    fn starts_with_a_fade_in_and_reaches_full_gain() {
        let (mut producer, mut stage, _, port) = stage(512);
        producer.push_entire_slice(&[0.5; 400]).unwrap();
        let mut out = vec![0.0; 400];
        assert_eq!(stage.process(&mut out), 200);
        // 60 ms at 1 kHz is 60 frames of ramp, heard after the DSP delay.
        assert!(out[0] < 0.01);
        assert!((out[2 * (100 + port.latency)] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn pause_fades_out_then_stops_reading() {
        let (mut producer, mut stage, shared, port) = stage(1024);
        producer.push_entire_slice(&[0.5; 2000]).unwrap();
        let mut out = vec![0.0; 200];
        stage.process(&mut out);
        shared.paused.store(true, Ordering::Release);
        let taken = stage.process(&mut out);
        assert_eq!(taken, 60, "fade-out length");
        let end = 60 + port.latency;
        assert!(out[2 * (end - 1)] < 0.01 && out[2 * end..].iter().all(|s| *s == 0.0));
        // Paused: nothing more is taken.
        assert_eq!(stage.process(&mut out), 0);
    }

    #[test]
    fn flush_discards_the_buffer_and_acknowledges() {
        let (mut producer, mut stage, shared, _) = stage(1024);
        producer.push_entire_slice(&[0.5; 2000]).unwrap();
        let mut out = vec![0.0; 200];
        stage.process(&mut out);
        shared.flush_request.fetch_add(1, Ordering::AcqRel);
        stage.process(&mut out);
        assert!(!shared.flush_pending());
        assert_eq!(stage.buffered_frames(), 0);
        assert_eq!(shared.consumed.load(Ordering::Acquire), 1000);
    }

    #[test]
    fn a_buffer_that_runs_dry_decays_instead_of_clicking() {
        let (mut producer, mut stage, shared, _) = stage(1024);
        // 100 frames at half scale, then nothing more.
        producer.push_entire_slice(&[0.5; 200]).unwrap();
        let mut out = vec![0.0; 400];
        stage.process(&mut out);
        let left: Vec<f32> = out.iter().step_by(2).copied().collect();
        let step = left
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .fold(0.0f32, f32::max);
        assert!(step < 0.03, "a jump of {step}");
        assert_eq!(left.last().copied(), Some(0.0));

        // A flush with nothing buffered still completes.
        shared.flush_request.fetch_add(1, Ordering::AcqRel);
        stage.process(&mut out);
        assert!(!shared.flush_pending());
    }

    #[test]
    fn settings_arrive_through_the_queue() {
        let (mut producer, mut stage, _, mut port) = stage(1024);
        let quiet = DspSettings {
            volume_db: -20.0,
            ..DspSettings::default()
        };
        port.updates.push(quiet.chain_params(RATE)).unwrap();
        producer.push_entire_slice(&[0.5; 2000]).unwrap();
        let mut out = vec![0.0; 2000];
        stage.process(&mut out);
        let settled = out[out.len() - 2];
        assert!(
            (settled - 0.05).abs() < 1e-3,
            "volume not applied: {settled}"
        );
    }

    #[test]
    fn the_tap_only_runs_while_analysis_is_on() {
        let (mut producer, mut stage, shared, mut port) = stage(1024);
        let mut tap = port.tap.take().unwrap();
        producer.push_entire_slice(&[0.5; 400]).unwrap();
        let mut out = vec![0.0; 200];
        stage.process(&mut out);
        assert_eq!(tap.slots(), 0);
        shared.analysis.store(true, Ordering::Release);
        stage.process(&mut out);
        assert_eq!(tap.slots(), 200);
        assert_eq!(tap.read_chunk(200).unwrap().as_slices().0, &out[..]);
    }
}
