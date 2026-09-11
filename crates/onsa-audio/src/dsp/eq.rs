//! The equaliser: graphic and parametric modes on one biquad stack
//! (SPEC §4.2).

use super::biquad::{Band, Coeffs, State};
use super::MAX_CHANNELS;

/// Most bands the EQ runs.
pub const MAX_BANDS: usize = 16;

/// Centre frequencies of the graphic EQ.
pub const GRAPHIC_FREQS: [f64; 10] = [
    31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

/// Q of every graphic band.
pub const GRAPHIC_Q: f64 = 1.41;

/// How the EQ is set up.
#[derive(Debug, Clone, PartialEq)]
pub enum EqMode {
    /// Ten fixed peaking bands; the values are their gains in dB.
    Graphic([f64; 10]),
    /// Free bands (at most [`MAX_BANDS`] are used).
    Parametric(Vec<Band>),
}

impl Default for EqMode {
    fn default() -> Self {
        Self::Graphic([0.0; 10])
    }
}

impl EqMode {
    /// The bands this mode runs, at most [`MAX_BANDS`].
    pub fn bands(&self) -> Vec<Band> {
        match self {
            Self::Graphic(gains) => GRAPHIC_FREQS
                .iter()
                .zip(gains)
                .map(|(&freq, &gain)| Band::peaking(freq, gain, GRAPHIC_Q))
                .collect(),
            Self::Parametric(bands) => bands.iter().take(MAX_BANDS).copied().collect(),
        }
    }
}

/// Coefficients of a whole EQ, fixed-size so it can travel to the callback
/// without allocating.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EqDesign {
    /// Coefficients of the used bands; the rest are identity.
    pub coeffs: [Coeffs; MAX_BANDS],
    /// Number of bands in use.
    pub count: usize,
}

impl EqDesign {
    /// A design that changes nothing.
    pub const FLAT: Self = Self {
        coeffs: [Coeffs::IDENTITY; MAX_BANDS],
        count: 0,
    };

    /// Designs every band for `sample_rate`. Identity bands are left out.
    pub fn new(bands: &[Band], sample_rate: u32) -> Self {
        let mut design = Self::FLAT;
        for band in bands.iter().take(MAX_BANDS) {
            let coeffs = Coeffs::design(band, f64::from(sample_rate));
            if coeffs != Coeffs::IDENTITY {
                design.coeffs[design.count] = coeffs;
                design.count += 1;
            }
        }
        design
    }

    /// Combined response at `freq`, in dB.
    pub fn response_db(&self, freq: f64, sample_rate: u32) -> f64 {
        self.coeffs[..self.count]
            .iter()
            .map(|coeffs| coeffs.magnitude_db(freq, f64::from(sample_rate)))
            .sum()
    }
}

/// The combined response on a logarithmic grid from 20 Hz to 20 kHz (or
/// just below Nyquist), as `(Hz, dB)` points, for drawing and auto preamp.
pub fn response_curve(bands: &[Band], sample_rate: u32, points: usize) -> Vec<(f64, f64)> {
    let design = EqDesign::new(bands, sample_rate);
    let top = 20_000f64.min(f64::from(sample_rate) * 0.49);
    let points = points.max(2);
    (0..points)
        .map(|i| {
            let freq = 20.0 * (top / 20.0).powf(i as f64 / (points - 1) as f64);
            (freq, design.response_db(freq, sample_rate))
        })
        .collect()
}

/// Automatic preamp: minus the highest boost of the combined curve, so the
/// EQ alone can never push a full-scale signal over 0 dBFS (SPEC §4.2).
pub fn auto_preamp_db(bands: &[Band], sample_rate: u32) -> f64 {
    let peak = response_curve(bands, sample_rate, 1024)
        .into_iter()
        .map(|(_, db)| db)
        .fold(0.0, f64::max);
    -peak
}

/// A running EQ: coefficients plus per-channel memory.
#[derive(Debug, Clone, Copy)]
pub struct EqBank {
    design: EqDesign,
    state: [[State; MAX_CHANNELS]; MAX_BANDS],
}

impl EqBank {
    /// A bank running `design`, with empty memory.
    pub fn new(design: EqDesign) -> Self {
        Self {
            design,
            state: [[State::default(); MAX_CHANNELS]; MAX_BANDS],
        }
    }

    /// The design being run.
    pub fn design(&self) -> &EqDesign {
        &self.design
    }

    /// A bank running `design` that starts from this bank's memory, so a
    /// change of settings does not restart the filters from silence.
    pub fn with_design(&self, design: EqDesign) -> Self {
        Self {
            design,
            state: self.state,
        }
    }

    /// Filters one sample of `channel`. Real-time safe.
    #[inline]
    pub fn process(&mut self, channel: usize, sample: f32) -> f32 {
        if channel >= MAX_CHANNELS {
            return sample;
        }
        let mut x = f64::from(sample);
        for band in 0..self.design.count {
            x = self.state[band][channel].process(&self.design.coeffs[band], x);
        }
        x as f32
    }

    /// Whether every filter's memory has decayed.
    pub fn is_quiet(&self) -> bool {
        self.state[..self.design.count]
            .iter()
            .all(|channels| channels.iter().all(State::is_quiet))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphic_mode_runs_ten_bands_at_the_fixed_frequencies() {
        let bands = EqMode::Graphic([1.0; 10]).bands();
        assert_eq!(bands.len(), 10);
        assert_eq!(bands[5].freq, 1000.0);
        assert!(bands.iter().all(|band| band.q == GRAPHIC_Q));
    }

    #[test]
    fn auto_preamp_cancels_the_highest_boost() {
        let bands = vec![
            Band::peaking(1000.0, 6.0, 1.0),
            Band::peaking(100.0, -4.0, 1.0),
        ];
        let preamp = auto_preamp_db(&bands, 48_000);
        assert!((preamp + 6.0).abs() < 0.1, "preamp {preamp}");
        // A curve that only cuts needs no preamp.
        assert_eq!(
            auto_preamp_db(&[Band::peaking(1000.0, -6.0, 1.0)], 48_000),
            0.0
        );
    }

    #[test]
    fn auto_preamp_keeps_a_boosted_full_scale_sine_below_clipping() {
        let bands = vec![Band::peaking(1000.0, 9.0, 1.41)];
        let preamp = super::super::db_to_gain(auto_preamp_db(&bands, 48_000) as f32);
        let mut bank = EqBank::new(EqDesign::new(&bands, 48_000));
        let mut peak = 0.0f32;
        for n in 0..48_000 {
            let x = (std::f32::consts::TAU * 1000.0 * n as f32 / 48_000.0).sin();
            peak = peak.max((bank.process(0, x) * preamp).abs());
        }
        assert!(peak <= 1.0 + 1e-3, "peak {peak}");
    }

    #[test]
    fn a_flat_bank_is_bit_exact() {
        let mut bank = EqBank::new(EqDesign::new(&EqMode::default().bands(), 48_000));
        for x in [0.0, 0.123_456, -0.9, 1.0] {
            assert_eq!(bank.process(0, x), x);
        }
    }
}
