use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// Pale, low-saturation, high-brightness icy blue-white with a subtle
/// deterministic shimmer built from two fast sine layers.
pub struct Ice;

impl Ice {
    pub fn new(_seed: u64) -> Self {
        Ice
    }
}

impl Animation for Ice {
    fn name(&self) -> &'static str {
        "ice"
    }

    fn description(&self) -> &'static str {
        "Pale, crisp icy shimmer"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.3);
        let shimmer = (time * std::f64::consts::TAU / period * 3.7).sin() * 0.06
            + (time * std::f64::consts::TAU / period * 1.3 + 0.5).sin() * 0.04;
        let hue = 195.0 + (time * std::f64::consts::TAU / (period * 5.0)).sin() * 8.0;
        let sat = context.params.saturation.clamp(0.12, 0.35);
        let brightness = (0.88 + shimmer).clamp(0.6, 1.0);
        let color = Hsv::new(hue, sat, 1.0).to_rgb();
        Frame::new(color, brightness as f32)
    }
}
