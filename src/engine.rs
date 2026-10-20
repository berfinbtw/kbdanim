//! The engine that drives one animation frame at a time: it turns real
//! elapsed time into speed-scaled simulated time, asks the current
//! animation for a `Frame`, and writes the result to the device
//! (subject to color quantization and the device's own write-diff
//! cache).

use crate::animation::{Animation, AnimationContext, AnimationParams};
use crate::brightness;
use crate::color::Color;
use crate::device::KeyboardDevice;
use crate::error::Result;
use crate::timing::FrameTimer;
use std::time::Instant;

pub struct EngineConfig {
    pub fps: u32,
    pub global_brightness: u8,
    /// Rounds each color channel to the nearest multiple of this value
    /// before writing, to avoid redundant near-identical sysfs writes
    /// for changes too small to perceive (spec section 42). `1` (or
    /// less) disables quantization.
    pub quantization: u8,
}

pub struct Engine {
    timer: FrameTimer,
    sim_time: f64,
    last_instant: Instant,
    config: EngineConfig,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        Engine {
            timer: FrameTimer::new(config.fps),
            sim_time: 0.0,
            last_instant: Instant::now(),
            config,
        }
    }

    pub fn set_fps(&mut self, fps: u32) {
        self.timer.set_fps(fps);
    }

    pub fn set_global_brightness(&mut self, brightness: u8) {
        self.config.global_brightness = brightness;
    }

    pub fn sim_time(&self) -> f64 {
        self.sim_time
    }

    /// Advances simulated time, computes the next frame, and writes it
    /// to `device`. Does **not** sleep; call `pace()` afterwards to
    /// maintain the configured frame rate.
    pub fn step(
        &mut self,
        animation: &mut dyn Animation,
        params: &AnimationParams,
        device: &mut dyn KeyboardDevice,
    ) -> Result<()> {
        let now = Instant::now();
        let real_delta = now.duration_since(self.last_instant).as_secs_f64().max(0.0);
        self.last_instant = now;
        let delta = real_delta * params.speed.max(0.0);
        self.sim_time += delta;

        let context = AnimationContext {
            params: params.clone(),
        };
        let frame = animation.update(self.sim_time, delta, &context);

        let color = quantize_color(frame.color, self.config.quantization);
        device.write_color(color)?;
        let final_brightness = brightness::combine(frame.brightness, self.config.global_brightness);
        device.write_brightness(final_brightness)?;
        Ok(())
    }

    /// Sleeps to maintain the configured frame rate. Call once per loop
    /// iteration, after `step()`.
    pub fn pace(&mut self) {
        self.timer.tick();
    }
}

fn quantize_color(c: Color, q: u8) -> Color {
    if q <= 1 {
        return c;
    }
    let q = q as i32;
    let round = |v: u8| -> u8 {
        let vi = v as i32;
        let r = ((vi + q / 2) / q) * q;
        r.clamp(0, 255) as u8
    };
    Color::new(round(c.r), round(c.g), round(c.b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::Frame;
    use crate::device::MockDevice;

    struct ConstantAnimation {
        color: Color,
    }
    impl Animation for ConstantAnimation {
        fn name(&self) -> &'static str {
            "const-test"
        }
        fn description(&self) -> &'static str {
            "test-only constant animation"
        }
        fn reset(&mut self) {}
        fn update(&mut self, _t: f64, _d: f64, _ctx: &AnimationContext) -> Frame {
            Frame::solid(self.color)
        }
    }

    #[test]
    fn step_writes_color_and_brightness() {
        let mut engine = Engine::new(EngineConfig {
            fps: 30,
            global_brightness: 200,
            quantization: 1,
        });
        let mut anim = ConstantAnimation {
            color: Color::new(10, 20, 30),
        };
        let params = AnimationParams::default();
        let mut device = MockDevice::new();
        engine.step(&mut anim, &params, &mut device).unwrap();
        assert_eq!(device.written_colors.last(), Some(&Color::new(10, 20, 30)));
        assert_eq!(device.written_brightness.last(), Some(&200));
    }

    #[test]
    fn quantize_rounds_to_nearest_multiple() {
        assert_eq!(quantize_color(Color::new(10, 10, 10), 1), Color::new(10, 10, 10));
        assert_eq!(quantize_color(Color::new(9, 12, 250), 5), Color::new(10, 10, 250));
    }

    #[test]
    fn quantize_clamps_to_valid_range() {
        let c = quantize_color(Color::new(254, 254, 254), 8);
        assert!(c.r <= 255 && c.g <= 255 && c.b <= 255);
    }
}
