use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// An acidic yellow-green pulse (attack/decay envelope, like `pulse`)
/// combined with a slow hue wobble for an unstable "toxic ooze" feel.
pub struct Toxic;

impl Toxic {
    pub fn new(_seed: u64) -> Self {
        Toxic
    }
}

impl Animation for Toxic {
    fn name(&self) -> &'static str {
        "toxic"
    }

    fn description(&self) -> &'static str {
        "Pulsing acid green-yellow with an unstable hue wobble"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.3);
        let t = time.rem_euclid(period) / period;
        let raw = if t < 0.3 {
            context.params.easing.apply(t / 0.3)
        } else {
            1.0 - context.params.easing.apply((t - 0.3) / 0.7)
        };
        let wobble = (time * std::f64::consts::TAU / (period * 2.7)).sin() * 10.0;
        let hue = 85.0 + wobble;
        let sat = context.params.saturation.clamp(0.85, 1.0);
        Frame::new(Hsv::new(hue, sat, 1.0).to_rgb(), (0.15 + raw * 0.85) as f32)
    }
}
