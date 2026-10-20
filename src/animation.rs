//! The animation trait and the types animations are built from.
//!
//! Every animation produces, for a given point in (speed-scaled) time, a
//! single [`Frame`]: a color plus an animation-local brightness. The
//! engine (see `engine.rs`) is responsible for turning that into actual
//! sysfs writes; animations never touch hardware directly.

use crate::color::Color;
use crate::easing::Easing;

/// A single computed frame: the color and animation-local brightness
/// (0.0 - 1.0) the animation wants at this instant. The engine multiplies
/// `brightness` by the user's global brightness before writing to the
/// device (see `brightness.rs`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub color: Color,
    pub brightness: f32,
}

impl Frame {
    pub fn new(color: Color, brightness: f32) -> Self {
        Frame {
            color,
            brightness: brightness.clamp(0.0, 1.0),
        }
    }

    /// A frame at full animation-local brightness.
    pub fn solid(color: Color) -> Self {
        Frame::new(color, 1.0)
    }
}

/// Parameters shared by (a subset of) all animations. Not every
/// animation uses every field; each only reads the ones relevant to it
/// (spec section 14).
#[derive(Debug, Clone)]
pub struct AnimationParams {
    /// Simulation speed multiplier; already folded into the `time`/
    /// `delta` passed to `Animation::update`, exposed here only for
    /// animations that want to additionally scale their own internal
    /// randomness cadence.
    pub speed: f64,
    pub color: Color,
    pub color2: Color,
    pub easing: Easing,
    /// The nominal duration (in seconds) of one cycle, pulse, or hold,
    /// depending on the animation.
    pub period: f64,
    /// 0.0-1.0 knob controlling how chaotic stochastic animations feel.
    pub randomness: f64,
    pub saturation: f64,
    /// 0.0-1.0 "warm/cool" knob used by a few animations (distinct from
    /// the `warm/cool` animation modes themselves).
    pub temperature: f64,
    /// General-purpose 0.0+ intensity multiplier (e.g. flicker speed).
    pub intensity: f64,
    /// Optional custom palette (spec sections 62-64); animations that
    /// support palettes fall back to `[color, color2]` when absent.
    pub palette: Option<Vec<Color>>,
    /// Seed for animations with internal randomness, enabling
    /// reproducible runs (spec section 61: `--seed`).
    pub seed: u64,
}

impl Default for AnimationParams {
    fn default() -> Self {
        AnimationParams {
            speed: 1.0,
            color: Color::new(0, 170, 255),
            color2: Color::new(255, 0, 128),
            easing: Easing::Smoothstep,
            period: 2.0,
            randomness: 0.5,
            saturation: 1.0,
            temperature: 0.5,
            intensity: 1.0,
            palette: None,
            seed: 0,
        }
    }
}

/// Read-only context passed to every animation on every frame.
pub struct AnimationContext {
    pub params: AnimationParams,
}

/// The common interface every animation implements. Animations are pure
/// with respect to hardware: they never touch sysfs, only compute a
/// `Frame` from elapsed time.
pub trait Animation: Send {
    /// Stable, kebab-case identifier used on the CLI (e.g. `"rainbow"`).
    fn name(&self) -> &'static str;

    /// One-line human-readable description, shown by `kbdanim list`.
    fn description(&self) -> &'static str;

    /// Resets any internal state (random walks, phase accumulators) back
    /// to a fresh, deterministic starting point.
    fn reset(&mut self);

    /// Computes the frame for the given point in time.
    ///
    /// - `time`: cumulative simulated seconds since the animation
    ///   started, already scaled by `speed` (i.e. this is *not* wall
    ///   clock time when speed != 1.0).
    /// - `delta`: simulated seconds since the previous call, also
    ///   speed-scaled. Useful for animations with internal random walks.
    /// - `context`: the current parameters (color, easing, etc).
    fn update(&mut self, time: f64, delta: f64, context: &AnimationContext) -> Frame;
}

/// Metadata about a registered animation, used by `kbdanim list`/`search`
/// and `kbdanim doctor`.
#[derive(Debug, Clone, Copy)]
pub struct AnimationInfo {
    pub name: &'static str,
    pub description: &'static str,
}

/// Tasteful default `(color, color2)` pairs for animations whose look
/// depends on `context.params.color`/`color2`. CLI flags (`--color`,
/// `--color2`) always override these when the user passes them
/// explicitly; this only supplies sensible defaults otherwise.
pub fn default_colors_for(mode: &str) -> (Color, Color) {
    match mode {
        "police" => (Color::new(255, 0, 0), Color::new(0, 70, 255)),
        "cyberpunk" => (Color::new(255, 0, 180), Color::new(0, 255, 255)),
        "warm" => (Color::new(255, 140, 20), Color::new(255, 60, 0)),
        "cool" => (Color::new(0, 140, 255), Color::new(0, 255, 220)),
        "night" => (Color::new(10, 10, 90), Color::new(70, 0, 100)),
        "toxic" => (Color::new(140, 255, 0), Color::new(80, 140, 0)),
        "meteor" => (Color::new(255, 255, 255), Color::new(20, 50, 255)),
        "lightning" => (Color::new(210, 225, 255), Color::new(0, 0, 20)),
        "sparkle" => (Color::new(255, 255, 255), Color::new(10, 10, 30)),
        "heartbeat" => (Color::new(255, 20, 60), Color::new(20, 0, 5)),
        "pulse" => (Color::new(255, 80, 80), Color::new(20, 0, 0)),
        "strobe" => (Color::new(255, 255, 255), Color::new(0, 0, 0)),
        _ => (Color::new(0, 170, 255), Color::new(255, 0, 128)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_clamps_brightness() {
        let f = Frame::new(Color::BLACK, 5.0);
        assert_eq!(f.brightness, 1.0);
        let f2 = Frame::new(Color::BLACK, -5.0);
        assert_eq!(f2.brightness, 0.0);
    }

    #[test]
    fn default_params_are_sane() {
        let p = AnimationParams::default();
        assert!(p.speed > 0.0);
        assert!((0.0..=1.0).contains(&p.saturation));
    }

    #[test]
    fn default_colors_cover_themed_modes() {
        let (c1, c2) = default_colors_for("police");
        assert_ne!(c1, c2);
    }
}
