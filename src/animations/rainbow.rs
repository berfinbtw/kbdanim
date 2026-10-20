use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// Continuous linear hue progression through the full color wheel at
/// full saturation/value. Distinct from `smooth-rainbow`, which eases
/// between discrete hue keyframes instead of moving at a constant rate.
pub struct Rainbow;

impl Rainbow {
    pub fn new(_seed: u64) -> Self {
        Rainbow
    }
}

impl Animation for Rainbow {
    fn name(&self) -> &'static str {
        "rainbow"
    }

    fn description(&self) -> &'static str {
        "Continuous rainbow hue cycle"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.5);
        let hue = (time / period) * 360.0;
        let sat = context.params.saturation.clamp(0.0, 1.0);
        Frame::solid(Hsv::new(hue, sat, 1.0).to_rgb())
    }
}
