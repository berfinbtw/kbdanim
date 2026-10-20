use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// Fast green flicker built from three sine waves at incommensurate
/// frequencies (deterministic pseudo-noise), evoking scrolling digital
/// code. Distinct from `fire`/`candle` (different hue family, much
/// higher flicker frequency, no persistent random-walk state).
pub struct Matrix;

impl Matrix {
    pub fn new(_seed: u64) -> Self {
        Matrix
    }
}

impl Animation for Matrix {
    fn name(&self) -> &'static str {
        "matrix"
    }

    fn description(&self) -> &'static str {
        "Fast flickering matrix-code green"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let rate = context.params.intensity.max(0.1) * 6.0;
        let n1 = (time * rate).sin();
        let n2 = (time * rate * 2.7 + 1.0).sin();
        let n3 = (time * rate * 4.3 + 2.0).sin();
        let noise = (n1 + n2 + n3) / 3.0;
        let brightness = (0.5 + noise * 0.45).clamp(0.05, 1.0);
        let sat = context.params.saturation.clamp(0.8, 1.0);
        Frame::new(Hsv::new(120.0, sat, 1.0).to_rgb(), brightness as f32)
    }
}
