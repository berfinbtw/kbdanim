use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::{hue_lerp, Hsv};

/// Glides between fixed 60-degree hue keyframes using the configured
/// easing curve, rather than rotating hue at a constant rate like
/// `rainbow`. The result eases in and out of each hue instead of moving
/// uniformly, which reads as calmer/smoother.
pub struct SmoothRainbow;

impl SmoothRainbow {
    pub fn new(_seed: u64) -> Self {
        SmoothRainbow
    }
}

impl Animation for SmoothRainbow {
    fn name(&self) -> &'static str {
        "smooth-rainbow"
    }

    fn description(&self) -> &'static str {
        "Eased rainbow that glides between hue keyframes"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(1.0);
        let step = time / period;
        let idx = step.floor();
        let frac = step - idx;
        let eased = context.params.easing.apply(frac);
        let hue1 = (idx * 60.0).rem_euclid(360.0);
        let hue2 = (hue1 + 60.0).rem_euclid(360.0);
        let hue = hue_lerp(hue1, hue2, eased);
        let sat = context.params.saturation.clamp(0.0, 1.0);
        Frame::solid(Hsv::new(hue, sat, 1.0).to_rgb())
    }
}
