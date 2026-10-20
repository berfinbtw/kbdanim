use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Color;

/// Switches instantly (no crossfade) between red, green, and blue (or a
/// custom palette). This is the classic hard-cut "RGB cycle" indicator
/// mode, deliberately distinct from `color-cycle` (smooth crossfade) and
/// `rainbow` (continuous hue motion).
pub struct RgbCycle;

impl RgbCycle {
    pub fn new(_seed: u64) -> Self {
        RgbCycle
    }
}

impl Animation for RgbCycle {
    fn name(&self) -> &'static str {
        "rgb-cycle"
    }

    fn description(&self) -> &'static str {
        "Hard-switching cycle through primary colors"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let palette: Vec<Color> = match &context.params.palette {
            Some(p) if p.len() >= 2 => p.clone(),
            _ => vec![Color::new(255, 0, 0), Color::new(0, 255, 0), Color::new(0, 0, 255)],
        };
        let n = palette.len() as i64;
        let period = context.params.period.max(0.2);
        let idx = (time / period).floor() as i64;
        let idx = idx.rem_euclid(n) as usize;
        Frame::solid(palette[idx])
    }
}
