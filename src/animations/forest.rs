use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// Deep, slowly shifting greens with an occasional gentle brightness dip
/// suggesting dappled light through leaves. Built from layered sines
/// (deterministic) rather than a random walk, giving a calmer, more
/// regular motion than `fire`/`candle`.
pub struct Forest;

impl Forest {
    pub fn new(_seed: u64) -> Self {
        Forest
    }
}

impl Animation for Forest {
    fn name(&self) -> &'static str {
        "forest"
    }

    fn description(&self) -> &'static str {
        "Slow shifting forest greens with dappled light"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let period = context.params.period.max(1.0) * 4.0;
        let hue_wave = (time * std::f64::consts::TAU / period).sin();
        let dapple = (time * std::f64::consts::TAU / (period * 0.22) + 0.7).sin();
        let hue = 100.0 + hue_wave * 18.0;
        let brightness = 0.6 + dapple * 0.3;
        let sat = context.params.saturation.clamp(0.55, 0.95);
        let color = Hsv::new(hue, sat, 0.9).to_rgb();
        Frame::new(color, brightness.clamp(0.15, 1.0) as f32)
    }
}
