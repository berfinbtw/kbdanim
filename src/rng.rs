//! A tiny, dependency-free pseudo-random number generator.
//!
//! kbdanim avoids the `rand` crate on purpose (see Cargo.toml). This is a
//! standard SplitMix64 generator: fast, deterministic given a seed, and
//! good enough for animation flicker/variation. It is NOT suitable for
//! cryptographic use, which kbdanim never needs.

#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // Avoid an all-zero state, which would otherwise produce a
        // degenerate (all-zero) first output.
        let seed = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
        Rng { state: seed }
    }

    /// Seeds from the current time, for non-deterministic use (e.g. the
    /// `kbdanim color random` one-shot command).
    pub fn from_entropy() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x2545F4914F6CDD1D);
        Rng::new(nanos)
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// Uniform float in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        // Use the top 53 bits to build a double with full mantissa
        // precision, a standard technique for u64 -> f64 in [0,1).
        let bits = self.next_u64() >> 11;
        (bits as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    /// Uniform float in [lo, hi).
    pub fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        if hi <= lo {
            return lo;
        }
        lo + self.next_f64() * (hi - lo)
    }

    /// Uniform integer in [lo, hi] (inclusive).
    pub fn range_u32(&mut self, lo: u32, hi: u32) -> u32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo) as u64 + 1;
        lo + (self.next_u64() % span) as u32
    }

    /// Returns true with the given probability (0.0 - 1.0).
    pub fn chance(&mut self, probability: f64) -> bool {
        self.next_f64() < probability.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_same_seed() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn next_f64_in_range() {
        let mut r = Rng::new(1234);
        for _ in 0..1000 {
            let v = r.next_f64();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn range_f64_bounds() {
        let mut r = Rng::new(7);
        for _ in 0..1000 {
            let v = r.range_f64(10.0, 20.0);
            assert!((10.0..20.0).contains(&v));
        }
    }

    #[test]
    fn range_u32_bounds() {
        let mut r = Rng::new(9);
        for _ in 0..1000 {
            let v = r.range_u32(5, 8);
            assert!((5..=8).contains(&v));
        }
    }

    #[test]
    fn zero_seed_is_not_degenerate() {
        let mut r = Rng::new(0);
        let v = r.next_u64();
        assert_ne!(v, 0);
    }
}
