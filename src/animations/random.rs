use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Color;
use crate::rng::Rng;

/// Smoothly interpolates between randomly chosen full-saturation colors
/// (spec section 60): never a single `rand()` jump cut, always a target
/// plus an eased glide towards it.
pub struct RandomColor {
    rng: Rng,
    current: Color,
    target: Color,
    elapsed_in_hold: f64,
    seed: u64,
}

impl RandomColor {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0xA5A5_A5A5_A5A5_A5A5);
        let current = Color::random_vivid(&mut rng);
        let target = Color::random_vivid(&mut rng);
        RandomColor {
            rng,
            current,
            target,
            elapsed_in_hold: 0.0,
            seed,
        }
    }
}

impl Animation for RandomColor {
    fn name(&self) -> &'static str {
        "random"
    }

    fn description(&self) -> &'static str {
        "Smoothly interpolated random colors"
    }

    fn reset(&mut self) {
        *self = RandomColor::new(self.seed);
    }

    fn update(&mut self, _time: f64, delta: f64, context: &AnimationContext) -> Frame {
        let hold = context.params.period.max(0.3);
        self.elapsed_in_hold += delta;
        let frac = (self.elapsed_in_hold / hold).min(1.0);
        let eased = context.params.easing.apply(frac);
        let color = Color::lerp(self.current, self.target, eased);
        if frac >= 1.0 {
            self.current = self.target;
            self.target = Color::random_vivid(&mut self.rng);
            self.elapsed_in_hold = 0.0;
        }
        Frame::solid(color)
    }
}
