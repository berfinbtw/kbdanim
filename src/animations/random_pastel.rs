use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Color;
use crate::rng::Rng;

/// Same smooth-interpolation mechanic as `random`, but drawing from a
/// soft, desaturated pastel palette instead of vivid fully-saturated hues.
pub struct RandomPastel {
    rng: Rng,
    current: Color,
    target: Color,
    elapsed_in_hold: f64,
    seed: u64,
}

impl RandomPastel {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x5A5A_5A5A_5A5A_5A5A);
        let current = Color::random_pastel(&mut rng);
        let target = Color::random_pastel(&mut rng);
        RandomPastel {
            rng,
            current,
            target,
            elapsed_in_hold: 0.0,
            seed,
        }
    }
}

impl Animation for RandomPastel {
    fn name(&self) -> &'static str {
        "random-pastel"
    }

    fn description(&self) -> &'static str {
        "Smoothly interpolated random pastel colors"
    }

    fn reset(&mut self) {
        *self = RandomPastel::new(self.seed);
    }

    fn update(&mut self, _time: f64, delta: f64, context: &AnimationContext) -> Frame {
        let hold = context.params.period.max(0.3) * 1.5;
        self.elapsed_in_hold += delta;
        let frac = (self.elapsed_in_hold / hold).min(1.0);
        let eased = context.params.easing.apply(frac);
        let color = Color::lerp(self.current, self.target, eased);
        if frac >= 1.0 {
            self.current = self.target;
            self.target = Color::random_pastel(&mut self.rng);
            self.elapsed_in_hold = 0.0;
        }
        Frame::solid(color)
    }
}
