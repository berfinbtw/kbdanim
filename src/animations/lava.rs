use super::util::SmoothRandom;
use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;
use crate::rng::Rng;

/// Deep red-orange glow, slower and darker than `fire`, with occasional
/// brief brightening "glow spikes" rather than continuous fast flicker.
pub struct Lava {
    hue_walk: SmoothRandom,
    base_glow: SmoothRandom,
    rng: Rng,
    spike_remaining: f64,
    seed: u64,
}

impl Lava {
    pub fn new(seed: u64) -> Self {
        Lava {
            hue_walk: SmoothRandom::new(seed, 0.0, 20.0, 0.6, 1.4),
            base_glow: SmoothRandom::new(seed ^ 0x4444_4444, 0.35, 0.6, 0.3, 0.8),
            rng: Rng::new(seed ^ 0x5555_5555),
            spike_remaining: 0.0,
            seed,
        }
    }
}

impl Animation for Lava {
    fn name(&self) -> &'static str {
        "lava"
    }

    fn description(&self) -> &'static str {
        "Slow deep-red lava glow with occasional bright spikes"
    }

    fn reset(&mut self) {
        *self = Lava::new(self.seed);
    }

    fn update(&mut self, _time: f64, delta: f64, context: &AnimationContext) -> Frame {
        let rate = context.params.intensity.max(0.1);
        let hue = self.hue_walk.advance(delta, rate * 0.6);
        let base = self.base_glow.advance(delta, rate);

        if self.spike_remaining > 0.0 {
            self.spike_remaining -= delta;
        } else if self.rng.chance(delta * 0.4 * rate) {
            self.spike_remaining = self.rng.range_f64(0.15, 0.4);
        }

        let brightness = if self.spike_remaining > 0.0 {
            (base + 0.4).min(1.0)
        } else {
            base
        };
        let sat = context.params.saturation.clamp(0.85, 1.0);
        let color = Hsv::new(hue, sat, 1.0).to_rgb();
        Frame::new(color, brightness as f32)
    }
}
