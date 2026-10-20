use crate::animation::{Animation, AnimationContext, Frame};

/// Continuous sine-wave brightness fade. Distinct from `pulse`, which
/// uses a sharp attack/decay envelope rather than a smooth sinusoid.
pub struct Breathing;

impl Breathing {
    pub fn new(_seed: u64) -> Self {
        Breathing
    }
}

impl Animation for Breathing {
    fn name(&self) -> &'static str {
        "breathing"
    }

    fn description(&self) -> &'static str {
        "Smooth sine-wave brightness breathing"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.2);
        let phase = (time / period) * std::f64::consts::TAU;
        let raw = phase.sin() * 0.5 + 0.5;
        let eased = context.params.easing.apply(raw);
        Frame::new(context.params.color, (0.04 + eased * 0.96) as f32)
    }
}
