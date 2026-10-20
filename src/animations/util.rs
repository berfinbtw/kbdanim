//! Small helpers shared by several animation implementations.

use crate::rng::Rng;

/// Smoothly glides towards randomly-chosen targets instead of jumping
/// between fresh random values every frame. This is what gives fire,
/// candle, lava, and similar animations their organic, non-jittery
/// flicker (spec section 60: "random target -> smooth interpolation ->
/// new target").
pub struct SmoothRandom {
    rng: Rng,
    current: f64,
    target: f64,
    progress: f64,
    hold: f64,
    lo: f64,
    hi: f64,
    hold_lo: f64,
    hold_hi: f64,
}

impl SmoothRandom {
    /// `lo`/`hi` bound the values produced. `hold_lo`/`hold_hi` bound how
    /// long (in simulated seconds) each glide towards a new target takes.
    pub fn new(seed: u64, lo: f64, hi: f64, hold_lo: f64, hold_hi: f64) -> Self {
        let mut rng = Rng::new(seed);
        let current = rng.range_f64(lo, hi);
        let target = rng.range_f64(lo, hi);
        let hold = rng.range_f64(hold_lo, hold_hi);
        SmoothRandom {
            rng,
            current,
            target,
            progress: 0.0,
            hold,
            lo,
            hi,
            hold_lo,
            hold_hi,
        }
    }

    /// Advances the walk by `delta` simulated seconds (optionally scaled
    /// by an extra `rate` multiplier) and returns the current value.
    pub fn advance(&mut self, delta: f64, rate: f64) -> f64 {
        let step = if self.hold > 1e-6 {
            delta * rate.max(0.0) / self.hold
        } else {
            1.0
        };
        self.progress += step;
        if self.progress >= 1.0 {
            self.current = self.target;
            self.target = self.rng.range_f64(self.lo, self.hi);
            self.hold = self.rng.range_f64(self.hold_lo, self.hold_hi);
            self.progress = 0.0;
        }
        let t = self.progress.clamp(0.0, 1.0);
        let eased = t * t * (3.0 - 2.0 * t); // local smoothstep, no crate dependency
        self.current + (self.target - self.current) * eased
    }

    /// Re-seeds the walk from scratch (used by `Animation::reset`).
    pub fn reseed(&mut self, seed: u64) {
        *self = SmoothRandom::new(seed, self.lo, self.hi, self.hold_lo, self.hold_hi);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stays_within_bounds() {
        let mut sr = SmoothRandom::new(1, 10.0, 20.0, 0.05, 0.2);
        for _ in 0..2000 {
            let v = sr.advance(0.01, 1.0);
            assert!((10.0..=20.0).contains(&v), "value out of bounds: {v}");
        }
    }

    #[test]
    fn deterministic_for_same_seed() {
        let mut a = SmoothRandom::new(42, 0.0, 1.0, 0.1, 0.3);
        let mut b = SmoothRandom::new(42, 0.0, 1.0, 0.1, 0.3);
        for _ in 0..500 {
            assert_eq!(a.advance(0.01, 1.0), b.advance(0.01, 1.0));
        }
    }
}
