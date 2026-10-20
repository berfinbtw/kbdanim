use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// Slow multi-color hue movement across greens, teals, and purples
/// (spec section 59's explicit description of aurora), built from two
/// sine waves at different frequencies so the hue drifts rather than
/// snapping between fixed colors like `color-cycle`.
pub struct Aurora;

impl Aurora {
    pub fn new(_seed: u64) -> Self {
        Aurora
    }
}

impl Animation for Aurora {
    fn name(&self) -> &'static str {
        "aurora"
    }

    fn description(&self) -> &'static str {
        "Slow multi-color aurora hue drift"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(1.0) * 6.0;
        let slow = (time * std::f64::consts::TAU / period).sin();
        let fast = (time * std::f64::consts::TAU / (period * 0.4) + 2.1).sin();
        // Center on green/teal/purple, spanning roughly 120-300 degrees.
        let hue = 210.0 + slow * 90.0 + fast * 15.0;
        let brightness = 0.55 + fast * 0.25;
        let sat = context.params.saturation.clamp(0.5, 0.9);
        let color = Hsv::new(hue, sat, 1.0).to_rgb();
        Frame::new(color, brightness.clamp(0.2, 1.0) as f32)
    }
}
