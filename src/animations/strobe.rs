use crate::animation::{Animation, AnimationContext, Frame};

/// Hard, un-eased on/off flashing at a configurable rate. Distinct from
/// `pulse`, which is a single color with a soft eased attack/decay.
pub struct Strobe;

impl Strobe {
    pub fn new(_seed: u64) -> Self {
        Strobe
    }
}

impl Animation for Strobe {
    fn name(&self) -> &'static str {
        "strobe"
    }

    fn description(&self) -> &'static str {
        "Hard on/off strobe flashing"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.05).min(2.0);
        let t = time.rem_euclid(period) / period;
        let on = t < 0.5;
        Frame::new(context.params.color, if on { 1.0 } else { 0.0 })
    }
}
