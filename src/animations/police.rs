use crate::animation::{Animation, AnimationContext, Frame};

/// Alternates between `color` and `color2` (red/blue by default, see
/// `default_colors_for`), each with a hard double-flash burst — the
/// classic police-light pattern. Distinct from `strobe` (single color,
/// even on/off) and `lightning` (irregular random spikes).
pub struct Police;

impl Police {
    pub fn new(_seed: u64) -> Self {
        Police
    }
}

impl Animation for Police {
    fn name(&self) -> &'static str {
        "police"
    }

    fn description(&self) -> &'static str {
        "Alternating red/blue police-light double flashes"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.4).min(3.0);
        let t = time.rem_euclid(period) / period;
        let (color, local) = if t < 0.5 {
            (context.params.color, t / 0.5)
        } else {
            (context.params.color2, (t - 0.5) / 0.5)
        };
        let on = (0.0..0.18).contains(&local) || (0.28..0.46).contains(&local);
        Frame::new(color, if on { 1.0 } else { 0.0 })
    }
}
