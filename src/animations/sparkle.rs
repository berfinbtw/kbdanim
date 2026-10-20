use crate::animation::{Animation, AnimationContext, Frame};
use crate::rng::Rng;

/// Frequent, brief bright twinkles on a dim base color — much more
/// frequent and lower-amplitude than `lightning`'s rare storm-like
/// spikes, and without any double-strike behavior.
pub struct Sparkle {
    rng: Rng,
    sparkle_remaining: f64,
    seed: u64,
}

impl Sparkle {
    pub fn new(seed: u64) -> Self {
        Sparkle {
            rng: Rng::new(seed ^ 0x9999_9999),
            sparkle_remaining: 0.0,
            seed,
        }
    }
}

impl Animation for Sparkle {
    fn name(&self) -> &'static str {
        "sparkle"
    }

    fn description(&self) -> &'static str {
        "Frequent brief twinkles on a dim base color"
    }

    fn reset(&mut self) {
        *self = Sparkle::new(self.seed);
    }

    fn update(&mut self, _time: f64, delta: f64, context: &AnimationContext) -> Frame {
        let rate = context.params.intensity.max(0.1);
        if self.sparkle_remaining > 0.0 {
            self.sparkle_remaining -= delta;
        } else if self.rng.chance(delta * 3.0 * rate) {
            self.sparkle_remaining = self.rng.range_f64(0.04, 0.09);
        }
        let brightness = if self.sparkle_remaining > 0.0 { 1.0 } else { 0.12 };
        Frame::new(context.params.color, brightness as f32)
    }
}
