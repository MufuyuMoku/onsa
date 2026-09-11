//! Fade curves for crossfade and micro-fade.

/// Shape of a crossfade (SPEC §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrossfadeCurve {
    /// Constant power: both gains are `cos`/`sin` of a quarter turn, so the
    /// midpoint sits at -3 dB each and the perceived loudness stays level.
    #[default]
    EqualPower,
    /// Straight lines: the midpoint sits at -6 dB each.
    Linear,
}

impl CrossfadeCurve {
    /// Gains `(outgoing, incoming)` at progress `t` in `0.0..=1.0`.
    pub fn gains(self, t: f32) -> (f32, f32) {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::EqualPower => {
                let angle = t * std::f32::consts::FRAC_PI_2;
                (angle.cos(), angle.sin())
            }
            Self::Linear => (1.0 - t, t),
        }
    }
}

/// Length of the micro-fade on pause, resume, seek and stop (SPEC §3.3 asks
/// for 50 to 100 ms).
pub const MICRO_FADE_SECONDS: f32 = 0.06;

/// Gain of a micro-fade at progress `t` in `0.0..=1.0`, rising from 0 to 1.
/// A raised cosine starts and ends with zero slope, so neither end of the
/// ramp is audible as a corner.
#[inline]
pub fn micro_fade_gain(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    0.5 - 0.5 * (t * std::f32::consts::PI).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_power_midpoint_is_minus_three_db() {
        let (out, inc) = CrossfadeCurve::EqualPower.gains(0.5);
        assert!((out - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert!((inc - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert!((out * out + inc * inc - 1.0).abs() < 1e-6);
    }

    #[test]
    fn curves_start_and_end_where_they_should() {
        for curve in [CrossfadeCurve::EqualPower, CrossfadeCurve::Linear] {
            let (out, inc) = curve.gains(0.0);
            assert!((out - 1.0).abs() < 1e-6 && inc.abs() < 1e-6);
            let (out, inc) = curve.gains(1.0);
            assert!(out.abs() < 1e-6 && (inc - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn micro_fade_is_smooth_at_both_ends() {
        assert_eq!(micro_fade_gain(0.0), 0.0);
        assert!((micro_fade_gain(1.0) - 1.0).abs() < 1e-6);
        assert!((micro_fade_gain(0.5) - 0.5).abs() < 1e-6);
    }
}
