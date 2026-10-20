use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// A cool cyan-blue hue wobble with its own independent, slower
/// brightness wobble. See `warm.rs` for the mirrored design.
pub struct Cool;

impl Cool {
    pub fn new(_seed: u64) -> Self {
        Cool
    }
}

impl Animation for Cool {
    fn name(&self) -> &'static str {
        "cool"
    }

    fn description(&self) -> &'static str {
        "Calm glow shifting through cool cyan-blue tones"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(1.0) * 5.0;
        let hue = 200.0 + (time * std::f64::consts::TAU / period).sin() * 20.0;
        let brightness =
            0.8 + (time * std::f64::consts::TAU / (period * 0.6) + 1.1).sin() * 0.15;
        let sat = context.params.saturation.clamp(0.6, 0.95);
        Frame::new(
            Hsv::new(hue, sat, 1.0).to_rgb(),
            brightness.clamp(0.45, 1.0) as f32,
        )
    }
}
