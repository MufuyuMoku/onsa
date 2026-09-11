//! The DSP chain (SPEC §4).
//!
//! ReplayGain is applied per track on the engine thread; EQ, preamp,
//! limiter, volume and dither run in the output callback, where changes are
//! heard at once. Everything here that runs in the callback is real-time
//! safe: buffers are allocated when a stage is built, never while it runs.

pub mod autoeq;
pub mod biquad;
pub mod chain;
pub mod dither;
pub mod eq;
pub mod limiter;
pub mod replaygain;

/// Channels the callback-side DSP processes; further channels pass through.
pub const MAX_CHANNELS: usize = 8;

/// Decibels to a linear gain factor.
#[inline]
pub fn db_to_gain(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

/// A linear gain factor to decibels.
#[inline]
pub fn gain_to_db(gain: f32) -> f32 {
    20.0 * gain.max(1e-12).log10()
}
