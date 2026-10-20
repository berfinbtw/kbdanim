use crate::animation::{Animation, AnimationContext, Frame};

pub struct StaticColor;

impl StaticColor {
    pub fn new(_seed: u64) -> Self {
        StaticColor
    }
}

impl Animation for StaticColor {
    fn name(&self) -> &'static str {
        "static"
    }

    fn description(&self) -> &'static str {
        "Static solid color"
    }

    fn reset(&mut self) {}

    fn update(&mut self, _time: f64, _delta: f64, context: &AnimationContext) -> Frame {
        Frame::solid(context.params.color)
    }
}
