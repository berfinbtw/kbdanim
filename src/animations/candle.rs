use super::util::SmoothRandom;
use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// A gentle amber flame: narrower hue band and much slower, lower-
/// amplitude flicker than `fire`, matching spec section 59's explicit
/// description of candle as "low-frequency random flicker".
pub struct Candle {
    flicker: SmoothRandom,
    hue_wobble: SmoothRandom,
    seed: u64,
}

impl Candle {
    pub fn new(seed: u64) -> Self {
        Candle {
            flicker: SmoothRandom::new(seed, 0.5, 0.85, 0.15, 0.5),
            hue_wobble: SmoothRandom::new(seed ^ 0x2222, 26.0, 38.0, 0.5, 1.5),
            seed,
        }
    }
}

impl Animation for Candle {
    fn name(&self) -> &'static str {
        "candle"
    }

    fn description(&self) -> &'static str {
        "Gentle amber candle flicker"
    }

    fn reset(&mut self) {
        self.flicker.reseed(self.seed);
        self.hue_wobble.reseed(self.seed ^ 0x2222);
    }

    fn update(&mut self, _time: f64, delta: f64, context: &AnimationContext) -> Frame {
        let rate = context.params.intensity.max(0.1);
        let hue = self.hue_wobble.advance(delta, rate);
        let brightness = self.flicker.advance(delta, rate);
        let color = Hsv::new(hue, 0.9, 1.0).to_rgb();
        Frame::new(color, brightness as f32)
    }
}
