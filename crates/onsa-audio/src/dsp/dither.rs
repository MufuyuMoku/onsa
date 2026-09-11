//! TPDF dither for integer output (SPEC §3.4, §4).
//!
//! When the device takes 16-bit (or narrower) integers, rounding the float
//! signal would leave correlated distortion in quiet passages. Adding
//! triangular noise of one LSB before the conversion turns it into a steady,
//! benign noise floor.

/// Triangular dither for a given integer width.
#[derive(Debug, Clone)]
pub struct Dither {
    lsb: f32,
    state: u32,
}

impl Dither {
    /// Dither for `bits`-bit integer output.
    pub fn new(bits: u32) -> Self {
        let bits = bits.clamp(2, 24);
        Self {
            lsb: 1.0 / (1u32 << (bits - 1)) as f32,
            state: 0x9E37_79B9,
        }
    }

    /// One LSB of the output format.
    pub fn lsb(&self) -> f32 {
        self.lsb
    }

    /// The next dither value, within ±1 LSB. Real-time safe.
    #[inline]
    pub fn sample(&mut self) -> f32 {
        (self.uniform() - self.uniform()) * self.lsb
    }

    #[inline]
    fn uniform(&mut self) -> f32 {
        // xorshift32: fast, allocation free and good enough for noise.
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        (x >> 8) as f32 / (1u32 << 24) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stays_within_one_lsb_with_the_triangular_variance() {
        let mut dither = Dither::new(16);
        let lsb = dither.lsb();
        let values: Vec<f32> = (0..200_000).map(|_| dither.sample()).collect();
        assert!(values.iter().all(|v| v.abs() <= lsb));
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;
        // TPDF of ±1 LSB has a variance of LSB²/6.
        let expected = lsb * lsb / 6.0;
        assert!(mean.abs() < lsb * 0.01, "mean {mean}");
        assert!(
            (variance / expected - 1.0).abs() < 0.05,
            "variance ratio {}",
            variance / expected
        );
    }
}
