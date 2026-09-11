//! Biquad filters from the RBJ Audio EQ Cookbook, state kept in `f64`
//! (SPEC §4.2).

use std::f64::consts::PI;

/// The filter shapes the EQ offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterKind {
    /// Bell around the centre frequency.
    Peaking,
    /// Boost or cut below the corner frequency.
    LowShelf,
    /// Boost or cut above the corner frequency.
    HighShelf,
    /// Passes below the corner frequency.
    LowPass,
    /// Passes above the corner frequency.
    HighPass,
    /// Removes a narrow band around the centre frequency.
    Notch,
}

impl FilterKind {
    /// Whether the gain setting changes this filter.
    pub fn uses_gain(self) -> bool {
        matches!(self, Self::Peaking | Self::LowShelf | Self::HighShelf)
    }
}

/// One EQ band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    /// Filter shape.
    pub kind: FilterKind,
    /// Centre or corner frequency in Hz.
    pub freq: f64,
    /// Gain in dB (peaking and shelves only).
    pub gain_db: f64,
    /// Quality factor (for shelves, the slope in the cookbook's Q form).
    pub q: f64,
    /// A disabled band passes the signal untouched.
    pub enabled: bool,
}

impl Band {
    /// A peaking band.
    pub fn peaking(freq: f64, gain_db: f64, q: f64) -> Self {
        Self {
            kind: FilterKind::Peaking,
            freq,
            gain_db,
            q,
            enabled: true,
        }
    }
}

/// Normalised biquad coefficients (`a0` = 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coeffs {
    /// Feed-forward coefficients.
    pub b0: f64,
    /// Feed-forward coefficients.
    pub b1: f64,
    /// Feed-forward coefficients.
    pub b2: f64,
    /// Feedback coefficients.
    pub a1: f64,
    /// Feedback coefficients.
    pub a2: f64,
}

impl Coeffs {
    /// Passes the signal untouched.
    pub const IDENTITY: Self = Self {
        b0: 1.0,
        b1: 0.0,
        b2: 0.0,
        a1: 0.0,
        a2: 0.0,
    };

    /// Designs a band for `sample_rate`. Out-of-range settings are clamped
    /// so a bad preset can never produce an unstable filter.
    pub fn design(band: &Band, sample_rate: f64) -> Self {
        if !band.enabled || sample_rate <= 0.0 {
            return Self::IDENTITY;
        }
        if band.kind.uses_gain() && band.gain_db.abs() < 1e-9 {
            return Self::IDENTITY;
        }
        let freq = band.freq.clamp(1.0, sample_rate * 0.49);
        let q = band.q.clamp(0.025, 40.0);
        let a = 10f64.powf(band.gain_db.clamp(-30.0, 30.0) / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        let root = 2.0 * a.sqrt() * alpha;

        let (b0, b1, b2, a0, a1, a2) = match band.kind {
            FilterKind::Peaking => (
                1.0 + alpha * a,
                -2.0 * cos,
                1.0 - alpha * a,
                1.0 + alpha / a,
                -2.0 * cos,
                1.0 - alpha / a,
            ),
            FilterKind::LowShelf => (
                a * ((a + 1.0) - (a - 1.0) * cos + root),
                2.0 * a * ((a - 1.0) - (a + 1.0) * cos),
                a * ((a + 1.0) - (a - 1.0) * cos - root),
                (a + 1.0) + (a - 1.0) * cos + root,
                -2.0 * ((a - 1.0) + (a + 1.0) * cos),
                (a + 1.0) + (a - 1.0) * cos - root,
            ),
            FilterKind::HighShelf => (
                a * ((a + 1.0) + (a - 1.0) * cos + root),
                -2.0 * a * ((a - 1.0) + (a + 1.0) * cos),
                a * ((a + 1.0) + (a - 1.0) * cos - root),
                (a + 1.0) - (a - 1.0) * cos + root,
                2.0 * ((a - 1.0) - (a + 1.0) * cos),
                (a + 1.0) - (a - 1.0) * cos - root,
            ),
            FilterKind::LowPass => (
                (1.0 - cos) / 2.0,
                1.0 - cos,
                (1.0 - cos) / 2.0,
                1.0 + alpha,
                -2.0 * cos,
                1.0 - alpha,
            ),
            FilterKind::HighPass => (
                (1.0 + cos) / 2.0,
                -(1.0 + cos),
                (1.0 + cos) / 2.0,
                1.0 + alpha,
                -2.0 * cos,
                1.0 - alpha,
            ),
            FilterKind::Notch => (1.0, -2.0 * cos, 1.0, 1.0 + alpha, -2.0 * cos, 1.0 - alpha),
        };
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Magnitude response at `freq`, in dB.
    pub fn magnitude_db(&self, freq: f64, sample_rate: f64) -> f64 {
        let w = 2.0 * PI * freq / sample_rate;
        let (s1, c1) = (-w).sin_cos();
        let (s2, c2) = (-2.0 * w).sin_cos();
        let num_re = self.b0 + self.b1 * c1 + self.b2 * c2;
        let num_im = self.b1 * s1 + self.b2 * s2;
        let den_re = 1.0 + self.a1 * c1 + self.a2 * c2;
        let den_im = self.a1 * s1 + self.a2 * s2;
        let num = num_re * num_re + num_im * num_im;
        let den = den_re * den_re + den_im * den_im;
        10.0 * (num.max(1e-300) / den.max(1e-300)).log10()
    }
}

/// Filter memory for one channel (transposed direct form II).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct State {
    z1: f64,
    z2: f64,
}

impl State {
    /// Filters one sample.
    #[inline]
    pub fn process(&mut self, c: &Coeffs, x: f64) -> f64 {
        let y = c.b0 * x + self.z1;
        self.z1 = c.b1 * x - c.a1 * y + self.z2;
        self.z2 = c.b2 * x - c.a2 * y;
        y
    }

    /// Whether the memory has decayed to nothing audible.
    #[inline]
    pub fn is_quiet(&self) -> bool {
        self.z1.abs() < 1e-9 && self.z2.abs() < 1e-9
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_1_SQRT_2;

    const RATE: f64 = 48_000.0;

    /// Runs a sine through the filter and measures the steady-state level.
    /// The amplitude comes from the RMS (times √2): at high frequencies few
    /// samples fall near the crest, so a sample peak would read low.
    fn measured_db(coeffs: &Coeffs, freq: f64) -> f64 {
        let mut state = State::default();
        let total = RATE as usize;
        let settle = total / 2;
        let mut sum = 0.0f64;
        for n in 0..total {
            let x = (2.0 * PI * freq * n as f64 / RATE).sin();
            let y = state.process(coeffs, x);
            if n >= settle {
                sum += y * y;
            }
        }
        let amplitude = (2.0 * sum / (total - settle) as f64).sqrt();
        20.0 * amplitude.max(1e-12).log10()
    }

    fn band(kind: FilterKind, freq: f64, gain_db: f64, q: f64) -> Band {
        Band {
            kind,
            freq,
            gain_db,
            q,
            enabled: true,
        }
    }

    #[test]
    fn measured_response_matches_the_computed_one_within_a_tenth_of_a_db() {
        let filters = [
            band(FilterKind::Peaking, 1000.0, 6.0, 1.41),
            band(FilterKind::Peaking, 125.0, -9.0, 0.7),
            band(FilterKind::LowShelf, 200.0, 5.0, 0.71),
            band(FilterKind::HighShelf, 6000.0, -4.0, 0.71),
            band(FilterKind::LowPass, 3000.0, 0.0, FRAC_1_SQRT_2),
            band(FilterKind::HighPass, 300.0, 0.0, FRAC_1_SQRT_2),
            band(FilterKind::Notch, 2000.0, 0.0, 4.0),
        ];
        for filter in filters {
            let coeffs = Coeffs::design(&filter, RATE);
            for freq in [40.0, 125.0, 400.0, 1000.0, 2500.0, 6000.0, 15000.0] {
                let computed = coeffs.magnitude_db(freq, RATE);
                if computed < -40.0 {
                    continue; // too deep to measure with a sine's peak
                }
                let measured = measured_db(&coeffs, freq);
                assert!(
                    (measured - computed).abs() < 0.1,
                    "{filter:?} at {freq} Hz: measured {measured:.3} dB, computed {computed:.3} dB"
                );
            }
        }
    }

    #[test]
    fn designs_hit_their_targets() {
        let peak = Coeffs::design(&band(FilterKind::Peaking, 1000.0, 6.0, 1.41), RATE);
        assert!((peak.magnitude_db(1000.0, RATE) - 6.0).abs() < 0.01);

        let low = Coeffs::design(&band(FilterKind::LowShelf, 1000.0, 6.0, 0.71), RATE);
        assert!((low.magnitude_db(20.0, RATE) - 6.0).abs() < 0.05);
        let high = Coeffs::design(&band(FilterKind::HighShelf, 1000.0, -6.0, 0.71), RATE);
        assert!((high.magnitude_db(20_000.0, RATE) + 6.0).abs() < 0.1);

        let low_pass = Coeffs::design(&band(FilterKind::LowPass, 2000.0, 0.0, FRAC_1_SQRT_2), RATE);
        assert!((low_pass.magnitude_db(2000.0, RATE) + 3.01).abs() < 0.01);
        let high_pass = Coeffs::design(
            &band(FilterKind::HighPass, 2000.0, 0.0, FRAC_1_SQRT_2),
            RATE,
        );
        assert!((high_pass.magnitude_db(2000.0, RATE) + 3.01).abs() < 0.01);

        let notch = Coeffs::design(&band(FilterKind::Notch, 2000.0, 0.0, 4.0), RATE);
        assert!(measured_db(&notch, 2000.0) < -40.0);
    }

    #[test]
    fn a_disabled_or_flat_band_is_the_identity() {
        let mut flat = band(FilterKind::Peaking, 1000.0, 0.0, 1.0);
        assert_eq!(Coeffs::design(&flat, RATE), Coeffs::IDENTITY);
        flat.gain_db = 6.0;
        flat.enabled = false;
        assert_eq!(Coeffs::design(&flat, RATE), Coeffs::IDENTITY);
    }

    #[test]
    fn wild_settings_stay_stable() {
        let wild = band(FilterKind::Peaking, 90_000.0, 80.0, 0.0);
        let coeffs = Coeffs::design(&wild, RATE);
        let mut state = State::default();
        let mut last = 0.0;
        for n in 0..RATE as usize {
            last = state.process(&coeffs, if n == 0 { 1.0 } else { 0.0 });
        }
        assert!(
            last.is_finite() && last.abs() < 1e-3,
            "impulse response did not decay: {last}"
        );
    }
}
