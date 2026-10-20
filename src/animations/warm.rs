use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// A warm amber-gold hue wobble combined with a slower independent
/// brightness wobble — two overlapping modulations at different
/// frequencies, unlike `breathing` (brightness only) or `oscillation`
/// (a plain two-color ping-pong).
pub struct Warm;

impl Warm {
    pub fn new(_seed: u64) -> Self {
        Warm
    }
}

impl Animation for Warm {
    fn name(&self) -> &'static str {
        "warm"
    }

    fn description(&self) -> &'static str {
        "Gentle glow shifting through warm amber-gold tones"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(1.0) * 4.0;
        let hue = 32.0 + (time * std::f64::consts::TAU / period).sin() * 12.0;
        let brightness =
            0.75 + (time * std::f64::consts::TAU / (period * 0.45) + 0.6).sin() * 0.2;
        let sat = context.params.saturation.clamp(0.7, 1.0);
        Frame::new(
            Hsv::new(hue, sat, 1.0).to_rgb(),
            brightness.clamp(0.4, 1.0) as f32,
        )
    }
}
