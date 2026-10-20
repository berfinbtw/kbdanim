//! Easing curves used to shape animation transitions so they feel smooth
//! instead of linear/robotic (see project spec section 9).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    Smoothstep,
    EaseIn,
    EaseOut,
    EaseInOut,
    Sine,
    Quadratic,
    Cubic,
}

impl Easing {
    /// Applies the curve to `t`, which is first clamped to [0, 1].
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::Smoothstep => t * t * (3.0 - 2.0 * t),
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::Sine => 0.5 - 0.5 * (std::f64::consts::PI * t).cos(),
            Easing::Quadratic => t * t,
            Easing::Cubic => t * t * t,
        }
    }

    pub fn parse(s: &str) -> Option<Easing> {
        match s.to_lowercase().as_str() {
            "linear" => Some(Easing::Linear),
            "smoothstep" => Some(Easing::Smoothstep),
            "ease-in" | "easein" | "ease_in" => Some(Easing::EaseIn),
            "ease-out" | "easeout" | "ease_out" => Some(Easing::EaseOut),
            "ease-in-out" | "easeinout" | "ease_in_out" => Some(Easing::EaseInOut),
            "sine" => Some(Easing::Sine),
            "quadratic" | "quad" => Some(Easing::Quadratic),
            "cubic" => Some(Easing::Cubic),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Easing::Linear => "linear",
            Easing::Smoothstep => "smoothstep",
            Easing::EaseIn => "ease-in",
            Easing::EaseOut => "ease-out",
            Easing::EaseInOut => "ease-in-out",
            Easing::Sine => "sine",
            Easing::Quadratic => "quadratic",
            Easing::Cubic => "cubic",
        }
    }

    pub fn all() -> &'static [Easing] {
        &[
            Easing::Linear,
            Easing::Smoothstep,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::Sine,
            Easing::Quadratic,
            Easing::Cubic,
        ]
    }
}

impl Default for Easing {
    fn default() -> Self {
        Easing::Smoothstep
    }
}

impl std::fmt::Display for Easing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_are_stable() {
        for e in Easing::all() {
            assert!((e.apply(0.0) - 0.0).abs() < 1e-9, "{e} at 0");
            assert!((e.apply(1.0) - 1.0).abs() < 1e-9, "{e} at 1");
        }
    }

    #[test]
    fn clamps_outside_range() {
        for e in Easing::all() {
            assert_eq!(e.apply(-1.0), e.apply(0.0));
            assert_eq!(e.apply(2.0), e.apply(1.0));
        }
    }

    #[test]
    fn parse_round_trip() {
        for e in Easing::all() {
            assert_eq!(Easing::parse(e.name()), Some(*e));
        }
    }

    #[test]
    fn parse_unknown_is_none() {
        assert_eq!(Easing::parse("not-a-real-easing"), None);
    }

    #[test]
    fn smoothstep_is_monotonic() {
        let mut last = -1.0;
        let mut t = 0.0;
        while t <= 1.0 {
            let v = Easing::Smoothstep.apply(t);
            assert!(v >= last);
            last = v;
            t += 0.05;
        }
    }
}
