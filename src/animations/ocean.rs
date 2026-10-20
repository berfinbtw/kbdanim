use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// Slow, smooth blue-teal hue and brightness swell built from two
/// layered sine waves at different frequencies, evoking a gentle rolling
/// sea rather than fire's sharp stochastic flicker.
pub struct Ocean;

impl Ocean {
    pub fn new(_seed: u64) -> Self {
        Ocean
    }
}

impl Animation for Ocean {
    fn name(&self) -> &'static str {
        "ocean"
    }

    fn description(&self) -> &'static str {
        "Slow rolling blue-teal ocean swell"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(1.0) * 3.0;
        let slow = (time * std::f64::consts::TAU / period).sin();
        let fast = (time * std::f64::consts::TAU / (period * 0.35) + 1.3).sin();
        let hue = 190.0 + slow * 12.0 + fast * 4.0;
        let brightness = 0.55 + (slow * 0.25 + fast * 0.1);
        let sat = context.params.saturation.clamp(0.6, 1.0);
        let color = Hsv::new(hue, sat, 1.0).to_rgb();
        Frame::new(color, brightness.clamp(0.15, 1.0) as f32)
    }
}
