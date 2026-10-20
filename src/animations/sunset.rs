use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Color;

/// Crossfades through a fixed golden-orange -> pink -> purple -> deep
/// blue gradient, like a sunset sky. Uses its own built-in palette
/// (unlike `color-cycle`, which is user-palette-driven) so it always
/// looks like a sunset regardless of `--color`.
pub struct Sunset;

impl Sunset {
    pub fn new(_seed: u64) -> Self {
        Sunset
    }
}

const PALETTE: [Color; 5] = [
    Color::new(255, 176, 59),
    Color::new(255, 111, 97),
    Color::new(219, 74, 122),
    Color::new(123, 63, 148),
    Color::new(35, 41, 122),
];

impl Animation for Sunset {
    fn name(&self) -> &'static str {
        "sunset"
    }

    fn description(&self) -> &'static str {
        "Crossfades through a golden sunset to deep purple gradient"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let n = PALETTE.len();
        let period = context.params.period.max(1.0) * 3.0;
        let step = (time / period).rem_euclid(n as f64);
        let idx = (step.floor() as usize).min(n - 1);
        let next = (idx + 1) % n;
        let frac = step - step.floor();
        let eased = context.params.easing.apply(frac);
        let color = Color::lerp(PALETTE[idx], PALETTE[next], eased);
        Frame::new(color, 0.85)
    }
}
