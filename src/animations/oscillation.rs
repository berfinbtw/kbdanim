use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Color;

/// A single sine wave ping-pongs between `color` and `color2` at
/// constant full brightness — the simplest possible two-color motion,
/// used as a building block distinct from `wave` (adds an independent
/// brightness harmonic) and `color-cycle` (arbitrary-length palettes).
pub struct Oscillation;

impl Oscillation {
    pub fn new(_seed: u64) -> Self {
        Oscillation
    }
}

impl Animation for Oscillation {
    fn name(&self) -> &'static str {
        "oscillation"
    }

    fn description(&self) -> &'static str {
        "Smoothly oscillates between two colors"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.3);
        let phase = (time * std::f64::consts::TAU / period).sin() * 0.5 + 0.5;
        let eased = context.params.easing.apply(phase);
        let color = Color::lerp(context.params.color, context.params.color2, eased);
        Frame::solid(color)
    }
}
