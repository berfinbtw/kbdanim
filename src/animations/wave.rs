use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Color;

/// Two sine waves of different frequency drive both a color crossfade
/// between `color`/`color2` and an independent brightness swell,
/// producing a layered "wave" motion distinct from the single-sine
/// `oscillation` (color only, constant brightness) or `ocean` (hue
/// shift within a fixed blue-teal band).
pub struct Wave;

impl Wave {
    pub fn new(_seed: u64) -> Self {
        Wave
    }
}

impl Animation for Wave {
    fn name(&self) -> &'static str {
        "wave"
    }

    fn description(&self) -> &'static str {
        "Layered sine-wave brightness and color motion"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.5);
        let w1 = (time * std::f64::consts::TAU / period).sin();
        let w2 = (time * std::f64::consts::TAU / (period * 0.37) + 0.8).sin();
        let combined = ((w1 * 0.65 + w2 * 0.35) + 1.0) / 2.0;
        let eased = context.params.easing.apply(combined.clamp(0.0, 1.0));
        let color_mix = w1 * 0.5 + 0.5;
        let color = Color::lerp(context.params.color, context.params.color2, color_mix);
        Frame::new(color, (0.1 + eased * 0.9) as f32)
    }
}
