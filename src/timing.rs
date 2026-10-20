//! Frame pacing built on `std::time::Instant` (monotonic). Deliberately
//! avoids the naive `loop { update(); sleep(16ms); }` pattern, which
//! accumulates drift because it ignores how long `update()` itself took.
//! Instead the next deadline always advances by a fixed frame duration
//! from the *previous* deadline, so occasional scheduling jitter does
//! not accumulate over a long-running daemon.

use std::time::{Duration, Instant};

pub struct FrameTimer {
    frame_duration: Duration,
    start: Instant,
    next_deadline: Instant,
}

impl FrameTimer {
    pub fn new(fps: u32) -> Self {
        let fps = fps.max(1);
        let frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
        let now = Instant::now();
        FrameTimer {
            frame_duration,
            start: now,
            next_deadline: now + frame_duration,
        }
    }

    pub fn set_fps(&mut self, fps: u32) {
        let fps = fps.max(1);
        self.frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
    }

    /// Seconds elapsed since this timer was created.
    pub fn elapsed(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Sleeps until the next frame deadline. If the caller fell far
    /// behind schedule (e.g. the process was suspended, or a very slow
    /// frame took multiple periods), the deadline is resynced to "now"
    /// instead of firing a burst of catch-up frames.
    pub fn tick(&mut self) {
        let now = Instant::now();
        if self.next_deadline > now {
            std::thread::sleep(self.next_deadline - now);
        }
        self.next_deadline += self.frame_duration;

        let now2 = Instant::now();
        if self.next_deadline + self.frame_duration < now2 {
            self.next_deadline = now2 + self.frame_duration;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_take_roughly_one_frame_period() {
        let mut timer = FrameTimer::new(200); // 5ms/frame, keeps the test fast
        let start = Instant::now();
        for _ in 0..10 {
            timer.tick();
        }
        let elapsed = start.elapsed().as_secs_f64();
        // 10 frames at 5ms should take at least ~40ms; generous upper
        // bound to keep this robust on loaded CI machines.
        assert!(elapsed >= 0.03, "elapsed too small: {elapsed}");
        assert!(elapsed <= 2.0, "elapsed too large: {elapsed}");
    }

    #[test]
    fn set_fps_changes_frame_duration() {
        let mut timer = FrameTimer::new(30);
        assert!((timer.frame_duration.as_secs_f64() - 1.0 / 30.0).abs() < 1e-9);
        timer.set_fps(60);
        assert!((timer.frame_duration.as_secs_f64() - 1.0 / 60.0).abs() < 1e-9);
    }

    #[test]
    fn zero_fps_is_clamped_to_one() {
        let timer = FrameTimer::new(0);
        assert!((timer.frame_duration.as_secs_f64() - 1.0).abs() < 1e-9);
    }
}
