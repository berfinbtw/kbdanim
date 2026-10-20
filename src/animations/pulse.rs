use crate::animation::{Animation, AnimationContext, Frame};

/// A sharp rhythmic pulse: fast attack, slower decay, repeating every
/// `period` seconds. Distinct from `breathing`'s smooth continuous sine.
pub struct Pulse;

impl Pulse {
    pub fn new(_seed: u64) -> Self {
        Pulse
    }
}

impl Animation for Pulse {
    fn name(&self) -> &'static str {
        "pulse"
    }

    fn description(&self) -> &'static str {
        "Sharp rhythmic brightness pulse"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.15);
        let t = (time.rem_euclid(period)) / period;
        let attack_end = 0.15;
        let raw = if t < attack_end {
            context.params.easing.apply(t / attack_end)
        } else {
            1.0 - context.params.easing.apply((t - attack_end) / (1.0 - attack_end))
        };
        Frame::new(context.params.color, (0.05 + raw * 0.95) as f32)
    }
}
