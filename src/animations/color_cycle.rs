use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Color;

/// Crossfades smoothly through a palette of arbitrary colors (falling
/// back to `[color, color2]` if no palette is configured). Distinct from
/// `rainbow`/`smooth-rainbow`, which move through hue space rather than
/// an arbitrary user-supplied color list, and from `rgb-cycle`, which
/// switches instantly rather than crossfading.
pub struct ColorCycle;

impl ColorCycle {
    pub fn new(_seed: u64) -> Self {
        ColorCycle
    }
}

impl Animation for ColorCycle {
    fn name(&self) -> &'static str {
        "color-cycle"
    }

    fn description(&self) -> &'static str {
        "Crossfades smoothly through a palette of colors"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let palette: Vec<Color> = match &context.params.palette {
            Some(p) if p.len() >= 2 => p.clone(),
            _ => vec![context.params.color, context.params.color2],
        };
        let n = palette.len();
        let period = context.params.period.max(0.2);
        let step = (time / period).rem_euclid(n as f64);
        let idx = (step.floor() as usize).min(n - 1);
        let next = (idx + 1) % n;
        let frac = step - step.floor();
        let eased = context.params.easing.apply(frac);
        let color = Color::lerp(palette[idx], palette[next], eased);
        Frame::solid(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::AnimationParams;

    #[test]
    fn falls_back_to_color_color2_without_palette() {
        let mut anim = ColorCycle::new(0);
        let params = AnimationParams::default();
        let ctx = AnimationContext { params };
        let f = anim.update(0.0, 0.0, &ctx);
        assert_eq!(f.color, ctx.params.color);
    }
}
