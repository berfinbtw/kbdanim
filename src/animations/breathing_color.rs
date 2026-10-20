use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// Combines `breathing`'s brightness sine with a slow, continuous
/// (non-oscillating) hue rotation like `rainbow`'s, but running roughly
/// an order of magnitude slower and driven by an independent period —
/// the two effects are layered rather than either one alone.
pub struct BreathingColor;

impl BreathingColor {
    pub fn new(_seed: u64) -> Self {
        BreathingColor
    }
}

impl Animation for BreathingColor {
    fn name(&self) -> &'static str {
        "breathing-color"
    }

    fn description(&self) -> &'static str {
        "Breathing brightness combined with a slow hue rotation"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let hue_period = context.params.period.max(0.5) * 15.0;
        let hue = (time / hue_period) * 360.0;

        let breath_period = context.params.period.max(0.2) * 2.0;
        let raw = (time * std::f64::consts::TAU / breath_period).sin() * 0.5 + 0.5;
        let eased = context.params.easing.apply(raw);

        let sat = context.params.saturation.clamp(0.0, 1.0);
        Frame::new(Hsv::new(hue, sat, 1.0).to_rgb(), (0.05 + eased * 0.95) as f32)
    }
}
