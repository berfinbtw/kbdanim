use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// A very low, capped brightness ceiling with a slow blue-violet hue
/// wobble — meant to be unobtrusive in a dark room (spec section 31).
pub struct Night;

impl Night {
    pub fn new(_seed: u64) -> Self {
        Night
    }
}

impl Animation for Night {
    fn name(&self) -> &'static str {
        "night"
    }

    fn description(&self) -> &'static str {
        "Very low brightness calm blue-violet glow for nighttime use"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(1.0) * 8.0;
        let hue = 255.0 + (time * std::f64::consts::TAU / period).sin() * 20.0;
        let breathing = 0.5 + (time * std::f64::consts::TAU / (period * 0.5)).sin() * 0.5;
        let sat = context.params.saturation.clamp(0.5, 0.9);
        // Night mode intentionally caps brightness well below normal
        // animations, since its entire purpose is to stay unobtrusive.
        let brightness = (0.08 + breathing * 0.12).clamp(0.03, 0.25);
        Frame::new(Hsv::new(hue, sat, 1.0).to_rgb(), brightness as f32)
    }
}
