use super::util::SmoothRandom;
use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// Warm red-orange-yellow palette combined with stochastic brightness
/// flicker (spec section 58's own example of the intended "single global
/// color + temporal modulation" technique for a single-zone keyboard).
pub struct Fire {
    hue_walk: SmoothRandom,
    flicker: SmoothRandom,
    seed: u64,
}

impl Fire {
    pub fn new(seed: u64) -> Self {
        Fire {
            hue_walk: SmoothRandom::new(seed, 0.0, 40.0, 0.3, 0.9),
            flicker: SmoothRandom::new(seed ^ 0x1111, 0.55, 1.0, 0.05, 0.18),
            seed,
        }
    }
}

impl Animation for Fire {
    fn name(&self) -> &'static str {
        "fire"
    }

    fn description(&self) -> &'static str {
        "Flickering fire simulation (red-orange-yellow)"
    }

    fn reset(&mut self) {
        self.hue_walk.reseed(self.seed);
        self.flicker.reseed(self.seed ^ 0x1111);
    }

    fn update(&mut self, _time: f64, delta: f64, context: &AnimationContext) -> Frame {
        let rate = context.params.intensity.max(0.1);
        let hue = self.hue_walk.advance(delta, rate);
        let brightness = self.flicker.advance(delta, rate * 2.0);
        let sat = context.params.saturation.clamp(0.85, 1.0);
        let color = Hsv::new(hue, sat, 1.0).to_rgb();
        Frame::new(color, brightness as f32)
    }
}
