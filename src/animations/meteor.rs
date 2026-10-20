use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Color;
use crate::rng::Rng;

/// A dark baseline punctuated by a bright flash that exponentially decays
/// into a cooler trail color, like a meteor streaking past, repeating at
/// a jittered interval. Distinct from `lightning` (much sharper, more
/// irregular, near-black baseline, occasional double-strikes) and from
/// `sparkle` (frequent small blips rather than one bright event at a time).
pub struct Meteor {
    rng: Rng,
    since_trigger: f64,
    next_interval: f64,
    seed: u64,
}

impl Meteor {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x3333_3333);
        let next_interval = rng.range_f64(1.5, 3.5);
        Meteor {
            rng,
            since_trigger: 0.0,
            next_interval,
            seed,
        }
    }
}

impl Animation for Meteor {
    fn name(&self) -> &'static str {
        "meteor"
    }

    fn description(&self) -> &'static str {
        "Periodic bright flash that decays into a trailing color"
    }

    fn reset(&mut self) {
        *self = Meteor::new(self.seed);
    }

    fn update(&mut self, _time: f64, delta: f64, context: &AnimationContext) -> Frame {
        self.since_trigger += delta;

        if self.since_trigger >= self.next_interval {
            self.since_trigger = 0.0;
            let base = context.params.period.max(0.5);
            let jitter = context.params.randomness.clamp(0.0, 1.0) * base * 0.6;
            self.next_interval = (base + self.rng.range_f64(-jitter, jitter)).max(0.3);
        }

        let decay_tau = (0.35 / context.params.intensity.max(0.1)).max(0.05);
        let brightness = (-self.since_trigger / decay_tau).exp().clamp(0.0, 1.0);
        let color = Color::lerp(context.params.color2, context.params.color, brightness);
        Frame::new(color, brightness as f32)
    }
}
