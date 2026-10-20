use crate::animation::{Animation, AnimationContext, Frame};
use crate::color::Hsv;

/// A classic "plasma" effect: several sine waves of incommensurate
/// frequency summed together and mapped onto hue, producing organic,
/// psychedelic motion without any randomness or discrete keyframes.
pub struct Plasma;

impl Plasma {
    pub fn new(_seed: u64) -> Self {
        Plasma
    }
}

impl Animation for Plasma {
    fn name(&self) -> &'static str {
        "plasma"
    }

    fn description(&self) -> &'static str {
        "Organic plasma field from layered sine waves"
    }

    fn reset(&mut self) {}

    fn update(&mut self, time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        let scale = context.params.intensity.max(0.1);
        let t = time * scale;
        let v = (t * 0.7).sin() + (t * 1.3 + 1.0).sin() + (t * 0.9 + 2.1).sin();
        let hue = ((v / 3.0) * 0.5 + 0.5) * 360.0;
        let sat = context.params.saturation.clamp(0.7, 1.0);
        Frame::solid(Hsv::new(hue, sat, 1.0).to_rgb())
    }
}
