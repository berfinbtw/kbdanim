use crate::animation::{Animation, AnimationContext, Frame};

/// Two quick beats followed by a rest, like a heartbeat. Distinct from
/// `pulse` (a single beat per cycle).
pub struct Heartbeat;

impl Heartbeat {
    pub fn new(_seed: u64) -> Self {
        Heartbeat
    }
}

impl Animation for Heartbeat {
    fn name(&self) -> &'static str {
        "heartbeat"
    }

    fn description(&self) -> &'static str {
        "Double-pulse heartbeat rhythm with a pause"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.6);
        let t = (time.rem_euclid(period)) / period;

        let beat = |center: f64, width: f64| -> f64 {
            let d = ((t - center) / width).abs();
            (1.0 - d).max(0.0)
        };

        let raw = beat(0.08, 0.08).max(beat(0.24, 0.11)).clamp(0.0, 1.0);
        let eased = context.params.easing.apply(raw);
        Frame::new(context.params.color, (0.03 + eased * 0.97) as f32)
    }
}
