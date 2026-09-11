//! The callback side of the DSP chain: EQ, preamp and limiter in a
//! reorderable order, then volume and dither, always last (SPEC §4).
//!
//! Settings are turned into [`ChainParams`] on the engine thread: a
//! fixed-size copyable value that travels to the callback through a
//! lock-free queue. [`Chain::set_params`] and [`Chain::process`] never
//! allocate, lock or block. Every change is smoothed so moving a slider is
//! never heard as a click: EQ changes crossfade between the old and the new
//! filter bank, gains glide with a 10 ms time constant, and the limiter only
//! releases smoothly.

use super::biquad::Band;
use super::dither::Dither;
use super::eq::{auto_preamp_db, EqBank, EqDesign, EqMode};
use super::limiter::{Limiter, DEFAULT_RELEASE_MS};
use super::replaygain::ReplayGainSettings;
use super::{db_to_gain, MAX_CHANNELS};

/// Length of the crossfade between two EQ settings.
const EQ_FADE_SECONDS: f32 = 0.02;
/// Time constant of preamp and volume changes.
const GAIN_SMOOTHING_SECONDS: f32 = 0.01;
/// Silence after which a settled chain stops working (SPEC §13.1).
const QUIET_SECONDS: f32 = 1.0;

/// A callback stage whose position can be changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// The equaliser.
    Eq,
    /// The preamp.
    Preamp,
    /// The peak limiter.
    Limiter,
}

/// The default order (SPEC §4).
pub const DEFAULT_ORDER: [Stage; 3] = [Stage::Eq, Stage::Preamp, Stage::Limiter];

/// Everything the user can set about the DSP chain.
#[derive(Debug, Clone, PartialEq)]
pub struct DspSettings {
    /// ReplayGain, applied per track before the chain.
    pub replaygain: ReplayGainSettings,
    /// Whether the EQ runs.
    pub eq_enabled: bool,
    /// EQ mode and bands.
    pub eq: EqMode,
    /// Manual preamp in dB, used when `auto_preamp` is off.
    pub preamp_db: f32,
    /// Preamp from the EQ curve (SPEC §4.2).
    pub auto_preamp: bool,
    /// Whether the limiter limits. Its look-ahead delay stays either way.
    pub limiter_enabled: bool,
    /// Limiter release in milliseconds.
    pub limiter_release_ms: f32,
    /// Output volume in dB (0 is unity).
    pub volume_db: f32,
    /// Add TPDF dither when the device takes 16-bit or narrower integers.
    pub dither: bool,
    /// Order of EQ, preamp and limiter.
    pub order: [Stage; 3],
}

impl Default for DspSettings {
    fn default() -> Self {
        Self {
            replaygain: ReplayGainSettings::default(),
            eq_enabled: false,
            eq: EqMode::default(),
            preamp_db: 0.0,
            auto_preamp: false,
            limiter_enabled: true,
            limiter_release_ms: DEFAULT_RELEASE_MS,
            volume_db: 0.0,
            dither: true,
            order: DEFAULT_ORDER,
        }
    }
}

impl DspSettings {
    /// The EQ bands in effect (none when the EQ is off).
    pub fn active_bands(&self) -> Vec<Band> {
        if self.eq_enabled {
            self.eq.bands()
        } else {
            Vec::new()
        }
    }

    /// The preamp in effect, in dB.
    pub fn effective_preamp_db(&self, sample_rate: u32) -> f32 {
        if self.auto_preamp {
            auto_preamp_db(&self.active_bands(), sample_rate) as f32
        } else {
            self.preamp_db
        }
    }

    /// What the callback needs at `sample_rate`.
    pub fn chain_params(&self, sample_rate: u32) -> ChainParams {
        ChainParams {
            eq: EqDesign::new(&self.active_bands(), sample_rate),
            preamp: db_to_gain(self.effective_preamp_db(sample_rate).clamp(-30.0, 30.0)),
            limiter_enabled: self.limiter_enabled,
            limiter_release_ms: self.limiter_release_ms.clamp(1.0, 2000.0),
            volume: if self.volume_db <= -90.0 {
                0.0
            } else {
                db_to_gain(self.volume_db.min(12.0))
            },
            dither: self.dither,
            order: self.order,
        }
    }
}

/// The chain's settings in callback form. Fixed size, so it crosses the
/// lock-free queue without allocating.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChainParams {
    /// EQ coefficients ([`EqDesign::FLAT`] when the EQ is off).
    pub eq: EqDesign,
    /// Linear preamp gain.
    pub preamp: f32,
    /// Whether the limiter limits.
    pub limiter_enabled: bool,
    /// Limiter release in milliseconds.
    pub limiter_release_ms: f32,
    /// Linear volume.
    pub volume: f32,
    /// Whether dither is allowed.
    pub dither: bool,
    /// Order of EQ, preamp and limiter.
    pub order: [Stage; 3],
}

/// A gain that glides to its target through two cascaded one-pole stages
/// (critically damped). Unlike a single pole, the glide starts with zero
/// slope, so the gain curve has no corner anywhere.
#[derive(Debug, Clone, Copy)]
struct Smoothed {
    stage: f32,
    current: f32,
    target: f32,
    coef: f32,
}

impl Smoothed {
    fn new(value: f32, sample_rate: u32) -> Self {
        // Two stages of half the time constant each.
        let frames = (GAIN_SMOOTHING_SECONDS * 0.5 * sample_rate as f32).max(1.0);
        Self {
            stage: value,
            current: value,
            target: value,
            coef: 1.0 - (-1.0 / frames).exp(),
        }
    }

    #[inline]
    fn next(&mut self) -> f32 {
        if self.current != self.target || self.stage != self.target {
            self.stage += (self.target - self.stage) * self.coef;
            self.current += (self.stage - self.current) * self.coef;
            if (self.target - self.stage).abs() < 1e-7 && (self.target - self.current).abs() < 1e-7
            {
                self.stage = self.target;
                self.current = self.target;
            }
        }
        self.current
    }

    fn settled(&self) -> bool {
        self.current == self.target && self.stage == self.target
    }
}

/// The running callback-side chain for one output.
pub struct Chain {
    channels: usize,
    order: [Stage; 3],
    eq: EqBank,
    /// The bank being faded in, while an EQ change is under way.
    eq_next: Option<EqBank>,
    eq_fade_left: u32,
    eq_fade_len: u32,
    preamp: Smoothed,
    volume: Smoothed,
    limiter: Limiter,
    dither: Option<Dither>,
    dither_on: bool,
    quiet_frames: u32,
    quiet_after: u32,
}

impl Chain {
    /// Builds the chain for an output. `integer_bits` is the width of an
    /// integer output format, `None` for float output (no dither needed).
    pub fn new(
        sample_rate: u32,
        channels: usize,
        integer_bits: Option<u32>,
        params: ChainParams,
    ) -> Self {
        let mut limiter = Limiter::new(sample_rate, channels);
        limiter.set_enabled(params.limiter_enabled);
        limiter.set_release_ms(params.limiter_release_ms);
        let dither = integer_bits.filter(|bits| *bits <= 16).map(Dither::new);
        Self {
            channels: channels.max(1),
            order: params.order,
            eq: EqBank::new(params.eq),
            eq_next: None,
            eq_fade_left: 0,
            eq_fade_len: ((EQ_FADE_SECONDS * sample_rate as f32) as u32).max(1),
            preamp: Smoothed::new(params.preamp, sample_rate),
            volume: Smoothed::new(params.volume, sample_rate),
            limiter,
            dither_on: params.dither && dither.is_some(),
            dither,
            quiet_frames: 0,
            quiet_after: (QUIET_SECONDS * sample_rate as f32) as u32,
        }
    }

    /// Delay the chain adds, in frames (the limiter's look-ahead).
    pub fn latency(&self) -> usize {
        self.limiter.latency()
    }

    /// Applies new settings. Real-time safe.
    pub fn set_params(&mut self, params: ChainParams) {
        let target = match &self.eq_next {
            Some(next) => next.design(),
            None => self.eq.design(),
        };
        if *target != params.eq {
            match self.eq_next.as_mut() {
                // Already fading: steer the incoming bank to the newest
                // design and let the fade carry on.
                Some(next) => *next = next.with_design(params.eq),
                None => {
                    self.eq_next = Some(self.eq.with_design(params.eq));
                    self.eq_fade_left = self.eq_fade_len;
                }
            }
        }
        self.preamp.target = params.preamp;
        self.volume.target = params.volume;
        self.limiter.set_enabled(params.limiter_enabled);
        self.limiter.set_release_ms(params.limiter_release_ms);
        self.dither_on = params.dither && self.dither.is_some();
        self.order = params.order;
        self.quiet_frames = 0;
    }

    /// Processes interleaved frames in place. Returns whether the limiter
    /// reduced the gain anywhere in the block. Real-time safe.
    pub fn process(&mut self, buffer: &mut [f32]) -> bool {
        let channels = self.channels;
        let silent_input = buffer.iter().all(|sample| *sample == 0.0);
        if silent_input {
            let frames = (buffer.len() / channels) as u32;
            self.quiet_frames = self.quiet_frames.saturating_add(frames);
            if self.is_settled() && self.quiet_frames > self.quiet_after {
                // Nothing left ringing: leave the silence untouched.
                return false;
            }
        } else {
            self.quiet_frames = 0;
        }

        let used = channels.min(MAX_CHANNELS);
        let mut limiting = false;
        let mut frame = [0.0f32; MAX_CHANNELS];
        for chunk in buffer.chunks_exact_mut(channels) {
            frame[..used].copy_from_slice(&chunk[..used]);
            let preamp = self.preamp.next();
            for stage in self.order {
                match stage {
                    Stage::Eq => self.run_eq(&mut frame[..used]),
                    Stage::Preamp => {
                        for sample in &mut frame[..used] {
                            *sample *= preamp;
                        }
                    }
                    Stage::Limiter => limiting |= self.limiter.process_frame(&mut frame[..used]),
                }
            }
            let volume = self.volume.next();
            for (out, sample) in chunk[..used].iter_mut().zip(&frame[..used]) {
                let mut value = sample * volume;
                if self.dither_on {
                    if let Some(dither) = self.dither.as_mut() {
                        value += dither.sample();
                    }
                }
                *out = value;
            }
            self.advance_eq_fade();
        }
        limiting
    }

    #[inline]
    fn run_eq(&mut self, frame: &mut [f32]) {
        match self.eq_next.as_mut() {
            None => {
                for (channel, sample) in frame.iter_mut().enumerate() {
                    *sample = self.eq.process(channel, *sample);
                }
            }
            Some(next) => {
                let weight = 1.0 - self.eq_fade_left as f32 / self.eq_fade_len as f32;
                for (channel, sample) in frame.iter_mut().enumerate() {
                    let old = self.eq.process(channel, *sample);
                    let new = next.process(channel, *sample);
                    *sample = old + (new - old) * weight;
                }
            }
        }
    }

    #[inline]
    fn advance_eq_fade(&mut self) {
        if self.eq_next.is_some() {
            self.eq_fade_left = self.eq_fade_left.saturating_sub(1);
            if self.eq_fade_left == 0 {
                if let Some(next) = self.eq_next.take() {
                    self.eq = next;
                }
            }
        }
    }

    /// Whether nothing is left ringing in the EQ or waiting in the limiter's
    /// delay, so the output is silent from here on.
    pub fn is_silent(&self) -> bool {
        self.eq_next.is_none() && self.eq.is_quiet() && self.limiter.is_quiet()
    }

    /// Whether nothing is fading, gliding or ringing.
    fn is_settled(&self) -> bool {
        self.eq_next.is_none()
            && self.preamp.settled()
            && self.volume.settled()
            && self.eq.is_quiet()
            && self.limiter.is_quiet()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 48_000;

    fn sine(frames: usize, freq: f32, amplitude: f32) -> Vec<f32> {
        (0..frames)
            .flat_map(|n| {
                let v = (std::f32::consts::TAU * freq * n as f32 / RATE as f32).sin() * amplitude;
                [v, v]
            })
            .collect()
    }

    /// Largest second difference of the left channel: a sine's is bounded
    /// by its curvature, a click shows up as a spike far above it.
    fn max_curvature(samples: &[f32]) -> f32 {
        let left: Vec<f32> = samples.iter().step_by(2).copied().collect();
        left.windows(3)
            .map(|w| (w[2] - 2.0 * w[1] + w[0]).abs())
            .fold(0.0, f32::max)
    }

    fn settings() -> DspSettings {
        DspSettings {
            eq_enabled: true,
            limiter_enabled: false,
            ..DspSettings::default()
        }
    }

    #[test]
    fn moving_the_eq_while_playing_does_not_click() {
        let mut settings = settings();
        let mut chain = Chain::new(RATE, 2, None, settings.chain_params(RATE));
        let mut audio = sine(RATE as usize, 1000.0, 0.2);
        let (before, after) = audio.split_at_mut(RATE as usize);
        chain.process(before);
        // A drastic change: +12 dB at 1 kHz, -12 dB at 125 Hz.
        let mut gains = [0.0; 10];
        gains[5] = 12.0;
        gains[2] = -12.0;
        settings.eq = EqMode::Graphic(gains);
        chain.set_params(settings.chain_params(RATE));
        chain.process(after);

        let transition = &audio[RATE as usize..RATE as usize + 2 * 4800];
        let steady = &audio[audio.len() - 2 * 4800..];
        let (jump, calm) = (max_curvature(transition), max_curvature(steady));
        assert!(
            jump <= calm * 1.05,
            "click: curvature {jump} vs steady {calm}"
        );
        // And the change did take effect.
        let level = steady.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!((level - 0.2 * 3.98).abs() < 0.02, "level {level}");
    }

    #[test]
    fn volume_and_preamp_glide() {
        let mut settings = settings();
        let mut chain = Chain::new(RATE, 2, None, settings.chain_params(RATE));
        let mut audio = sine(RATE as usize, 440.0, 0.5);
        let (before, after) = audio.split_at_mut(RATE as usize / 2);
        chain.process(before);
        settings.volume_db = -20.0;
        settings.preamp_db = 6.0;
        chain.set_params(settings.chain_params(RATE));
        chain.process(after);
        let steady_curvature = 0.5 * (std::f32::consts::TAU * 440.0 / RATE as f32).powi(2);
        // Skip the start: after the limiter's silent look-ahead the sine
        // begins at phase zero, a corner that is not a gain change.
        let start = 2 * (chain.latency() + 16);
        let jump = max_curvature(&audio[start..]);
        assert!(
            jump <= steady_curvature * 1.1,
            "click: {jump} vs {steady_curvature}"
        );
    }

    #[test]
    fn the_limiter_holds_the_ceiling_after_a_boost() {
        let settings = DspSettings {
            eq_enabled: true,
            preamp_db: 12.0,
            ..DspSettings::default()
        };
        let mut chain = Chain::new(RATE, 2, None, settings.chain_params(RATE));
        let mut audio = sine(RATE as usize, 1000.0, 0.9);
        assert!(chain.process(&mut audio));
        let peak = audio.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(peak <= db_to_gain(-0.1) + 1e-6, "peak {peak}");
    }

    #[test]
    fn default_chain_is_transparent_apart_from_its_delay() {
        let mut chain = Chain::new(RATE, 2, None, DspSettings::default().chain_params(RATE));
        let input = sine(4800, 440.0, 0.5);
        let mut audio = input.clone();
        chain.process(&mut audio);
        let latency = chain.latency() * 2;
        assert_eq!(&audio[latency..], &input[..input.len() - latency]);
    }

    #[test]
    fn auto_preamp_follows_the_curve() {
        let mut gains = [0.0; 10];
        gains[3] = 6.0;
        let settings = DspSettings {
            eq_enabled: true,
            eq: EqMode::Graphic(gains),
            auto_preamp: true,
            preamp_db: 3.0,
            ..DspSettings::default()
        };
        let preamp = settings.effective_preamp_db(RATE);
        assert!((preamp + 6.0).abs() < 0.2, "preamp {preamp}");
        let off = DspSettings {
            eq_enabled: false,
            ..settings
        };
        assert_eq!(off.effective_preamp_db(RATE), 0.0);
    }

    #[test]
    fn dither_only_for_narrow_integer_output() {
        let params = DspSettings::default().chain_params(RATE);
        let mut float = Chain::new(RATE, 2, None, params);
        let mut sixteen = Chain::new(RATE, 2, Some(16), params);
        let mut wide = Chain::new(RATE, 2, Some(24), params);
        let mut a = sine(960, 440.0, 0.25);
        let mut b = a.clone();
        let mut c = a.clone();
        float.process(&mut a);
        sixteen.process(&mut b);
        wide.process(&mut c);
        assert_eq!(a, c);
        assert_ne!(a, b);
        let lsb = 1.0 / 32768.0;
        assert!(a.iter().zip(&b).all(|(x, y)| (x - y).abs() <= lsb));
    }

    #[test]
    fn a_settled_chain_leaves_silence_alone() {
        let params = DspSettings::default().chain_params(RATE);
        let mut chain = Chain::new(RATE, 2, Some(16), params);
        let mut silence = vec![0.0f32; 2 * RATE as usize * 2];
        chain.process(&mut silence);
        let mut more = vec![0.0f32; 2 * 512];
        chain.process(&mut more);
        assert!(
            more.iter().all(|v| *v == 0.0),
            "dither noise leaked into digital silence"
        );
    }
}
