use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Color;
use crate::rng::Rng;

/// Ping-pongs between `color`/`color2` (neon magenta/cyan by default)
/// with brief randomized "glitch" brightness dips layered on top —
/// distinct from `oscillation`'s clean, glitch-free ping-pong.
pub struct Cyberpunk {
    rng: Rng,
    glitch_remaining: f64,
    seed: u64,
}

impl Cyberpunk {
    pub fn new(seed: u64) -> Self {
        Cyberpunk {
            rng: Rng::new(seed ^ 0x6666_6666),
            glitch_remaining: 0.0,
            seed,
        }
    }
}

impl Animation for Cyberpunk {
    fn name(&self) -> &'static str {
        "cyberpunk"
    }

    fn description(&self) -> &'static str {
        "Neon magenta/cyan crossfade with glitch flicker"
    }

    fn reset(&mut self) {
        *self = Cyberpunk::new(self.seed);
    }

    fn update(&mut self, time: f64, delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(0.5) * 2.0;
        let phase = (time / period).rem_euclid(1.0);
        let raw = if phase < 0.5 { phase * 2.0 } else { (1.0 - phase) * 2.0 };
        let eased = context.params.easing.apply(raw);
        let color = Color::lerp(context.params.color, context.params.color2, eased);

        if self.glitch_remaining > 0.0 {
            self.glitch_remaining -= delta;
        } else if self.rng.chance(delta * 0.6) {
            self.glitch_remaining = self.rng.range_f64(0.03, 0.09);
        }
        let brightness = if self.glitch_remaining > 0.0 { 0.15 } else { 1.0 };
        Frame::new(color, brightness as f32)
    }
}
