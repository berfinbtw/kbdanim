//! Brightness is deliberately split into two independent knobs (spec
//! section 10):
//!
//! - `animation brightness` (0.0 - 1.0): how bright *this frame* of the
//!   current animation wants to be (e.g. the dip of a breathing cycle).
//! - `global brightness` (0 - 255): the user's overall brightness dial.
//!
//! The value actually written to sysfs is their product.

/// Combines an animation-local brightness (0.0-1.0, clamped) with the
/// global brightness (0-255) to produce the final sysfs brightness value.
pub fn combine(animation_brightness: f32, global_brightness: u8) -> u8 {
    let a = animation_brightness.clamp(0.0, 1.0) as f64;
    let g = global_brightness as f64;
    (a * g).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_animation_brightness_passes_global_through() {
        assert_eq!(combine(1.0, 200), 200);
    }

    #[test]
    fn zero_animation_brightness_is_off() {
        assert_eq!(combine(0.0, 255), 0);
    }

    #[test]
    fn half_brightness_halves_global() {
        assert_eq!(combine(0.5, 200), 100);
    }

    #[test]
    fn out_of_range_animation_brightness_is_clamped() {
        assert_eq!(combine(2.0, 100), 100);
        assert_eq!(combine(-1.0, 100), 0);
    }

    #[test]
    fn zero_global_brightness_is_always_off() {
        assert_eq!(combine(1.0, 0), 0);
    }
}
