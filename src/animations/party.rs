use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Color;
use crate::rng::Rng;

/// Hard, instant cuts to a new random vivid color every ~0.1-0.3s —
/// energetic and punchy, unlike `random`'s smooth crossfades between
/// targets.
pub struct Party {
    rng: Rng,
    color: Color,
    elapsed: f64,
    hold: f64,
    seed: u64,
}

impl Party {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x7777_7777);
        let color = Color::random_vivid(&mut rng);
        let hold = rng.range_f64(0.12, 0.3);
        Party {
            rng,
            color,
            elapsed: 0.0,
            hold,
            seed,
        }
    }
}

impl Animation for Party {
    fn name(&self) -> &'static str {
        "party"
    }

    fn description(&self) -> &'static str {
        "Fast, punchy random color jumps"
    }

    fn reset(&mut self) {
        *self = Party::new(self.seed);
    }

    fn update(&mut self, _time: f64, delta: f64, context: &AnimationContext) -> Frame {
        self.elapsed += delta * context.params.intensity.max(0.2);
        if self.elapsed >= self.hold {
            self.elapsed = 0.0;
            self.color = Color::random_vivid(&mut self.rng);
            self.hold = self.rng.range_f64(0.10, 0.28);
        }
        Frame::solid(self.color)
    }
}
