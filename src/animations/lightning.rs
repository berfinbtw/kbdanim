use crate::animation::{Animation, AnimationContext, Frame};
use crate::rng::Rng;

/// A near-black baseline punctuated by short, sharp, irregular bright
/// spikes, occasionally striking twice in quick succession (spec
/// section 59: "dark baseline + short bright spikes"). Distinct from
/// `meteor` (a single exponentially-decaying glow per event, color
/// blends toward a trail color) and `sparkle` (much more frequent,
/// smaller, no double-strikes).
pub struct Lightning {
    rng: Rng,
    until_next: f64,
    flash_remaining: f64,
    flash_count: u8,
    seed: u64,
}

impl Lightning {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x8888_8888);
        let until_next = rng.range_f64(0.8, 2.5);
        Lightning {
            rng,
            until_next,
            flash_remaining: 0.0,
            flash_count: 0,
            seed,
        }
    }
}

impl Animation for Lightning {
    fn name(&self) -> &'static str {
        "lightning"
    }

    fn description(&self) -> &'static str {
        "Dark baseline with short, irregular lightning spikes"
    }

    fn reset(&mut self) {
        *self = Lightning::new(self.seed);
    }

    fn update(&mut self, _time: f64, delta: f64, context: &AnimationContext) -> Frame {
        let rate = context.params.intensity.max(0.1);

        if self.flash_remaining > 0.0 {
            self.flash_remaining -= delta * rate;
            return Frame::new(context.params.color, 1.0);
        }

        self.until_next -= delta * rate;
        if self.until_next <= 0.0 {
            self.flash_remaining = self.rng.range_f64(0.03, 0.08);
            self.flash_count += 1;
            if self.flash_count < 2 && self.rng.chance(0.5) {
                // Quick double-strike.
                self.until_next = self.rng.range_f64(0.05, 0.15);
            } else {
                self.flash_count = 0;
                let base = context.params.period.max(0.5);
                let jitter = context.params.randomness.clamp(0.0, 1.0);
                self.until_next = base * self.rng.range_f64(0.5, 1.0 + jitter);
            }
        }
        Frame::new(context.params.color, 0.02)
    }
}
