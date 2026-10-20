//! Central animation registry.
//!
//! Adding a new animation only requires: writing `animations/foo.rs`,
//! declaring `mod foo;` below, and adding one line to `FACTORIES`. No
//! other part of the engine needs to change (spec sections 71-72).

mod util;

mod aurora;
mod breathing;
mod breathing_color;
mod candle;
mod color_cycle;
mod cool;
mod cyberpunk;
mod fire;
mod forest;
mod heartbeat;
mod ice;
mod lava;
mod lightning;
mod matrix;
mod meteor;
mod night;
mod ocean;
mod oscillation;
mod party;
mod plasma;
mod police;
mod pulse;
mod random;
mod random_pastel;
mod rainbow;
mod rgb_cycle;
mod smooth_rainbow;
mod sparkle;
mod static_color;
mod strobe;
mod sunset;
mod toxic;
mod warm;
mod wave;

use crate::animation::{Animation, AnimationInfo};

type Factory = fn(u64) -> Box<dyn Animation>;

/// `(registry name, factory)` pairs. The registry name here is only used
/// for lookup; the authoritative display name/description come from
/// each animation's own `name()`/`description()` so the two can never
/// drift out of sync.
const FACTORIES: &[(&str, Factory)] = &[
    ("static", |seed| Box::new(static_color::StaticColor::new(seed))),
    ("breathing", |seed| Box::new(breathing::Breathing::new(seed))),
    ("pulse", |seed| Box::new(pulse::Pulse::new(seed))),
    ("heartbeat", |seed| Box::new(heartbeat::Heartbeat::new(seed))),
    ("strobe", |seed| Box::new(strobe::Strobe::new(seed))),
    ("rainbow", |seed| Box::new(rainbow::Rainbow::new(seed))),
    ("smooth-rainbow", |seed| {
        Box::new(smooth_rainbow::SmoothRainbow::new(seed))
    }),
    ("color-cycle", |seed| Box::new(color_cycle::ColorCycle::new(seed))),
    ("random", |seed| Box::new(random::RandomColor::new(seed))),
    ("random-pastel", |seed| {
        Box::new(random_pastel::RandomPastel::new(seed))
    }),
    ("breathing-color", |seed| {
        Box::new(breathing_color::BreathingColor::new(seed))
    }),
    ("warm", |seed| Box::new(warm::Warm::new(seed))),
    ("cool", |seed| Box::new(cool::Cool::new(seed))),
    ("night", |seed| Box::new(night::Night::new(seed))),
    ("fire", |seed| Box::new(fire::Fire::new(seed))),
    ("candle", |seed| Box::new(candle::Candle::new(seed))),
    ("ocean", |seed| Box::new(ocean::Ocean::new(seed))),
    ("forest", |seed| Box::new(forest::Forest::new(seed))),
    ("aurora", |seed| Box::new(aurora::Aurora::new(seed))),
    ("sunset", |seed| Box::new(sunset::Sunset::new(seed))),
    ("lava", |seed| Box::new(lava::Lava::new(seed))),
    ("ice", |seed| Box::new(ice::Ice::new(seed))),
    ("meteor", |seed| Box::new(meteor::Meteor::new(seed))),
    ("lightning", |seed| Box::new(lightning::Lightning::new(seed))),
    ("sparkle", |seed| Box::new(sparkle::Sparkle::new(seed))),
    ("oscillation", |seed| Box::new(oscillation::Oscillation::new(seed))),
    ("wave", |seed| Box::new(wave::Wave::new(seed))),
    ("plasma", |seed| Box::new(plasma::Plasma::new(seed))),
    ("cyberpunk", |seed| Box::new(cyberpunk::Cyberpunk::new(seed))),
    ("matrix", |seed| Box::new(matrix::Matrix::new(seed))),
    ("toxic", |seed| Box::new(toxic::Toxic::new(seed))),
    ("party", |seed| Box::new(party::Party::new(seed))),
    ("police", |seed| Box::new(police::Police::new(seed))),
    ("rgb-cycle", |seed| Box::new(rgb_cycle::RgbCycle::new(seed))),
];

/// Looks up an animation by its CLI name and constructs a fresh instance
/// seeded with `seed`. Returns `None` for unknown names.
pub fn create(name: &str, seed: u64) -> Option<Box<dyn Animation>> {
    for (n, factory) in FACTORIES.iter() {
        if *n == name {
            return Some((*factory)(seed));
        }
    }
    None
}

/// Lists every registered animation's name and description, in
/// registration order (used by `kbdanim list`, `kbdanim doctor`, shell
/// completions, and the man page generator).
pub fn list() -> Vec<AnimationInfo> {
    let mut result = Vec::with_capacity(FACTORIES.len());
    for (_, factory) in FACTORIES.iter() {
        let inst = (*factory)(0);
        result.push(AnimationInfo {
            name: inst.name(),
            description: inst.description(),
        });
    }
    result
}

/// Just the names, for validation and shell completion.
pub fn names() -> Vec<&'static str> {
    let mut result = Vec::with_capacity(FACTORIES.len());
    for (n, _) in FACTORIES.iter() {
        result.push(*n);
    }
    result
}

pub fn count() -> usize {
    FACTORIES.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_at_least_thirty_animations() {
        assert!(count() >= 30, "expected at least 30 animations, found {}", count());
    }

    #[test]
    fn every_registry_name_matches_its_animation_name() {
        for (registry_name, factory) in FACTORIES.iter() {
            let inst = (*factory)(0);
            assert_eq!(
                inst.name(),
                *registry_name,
                "registry key '{registry_name}' does not match Animation::name()"
            );
        }
    }

    #[test]
    fn every_name_is_unique() {
        let names = names();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(names.len(), sorted.len(), "duplicate animation names found");
    }

    #[test]
    fn create_known_name_succeeds() {
        assert!(create("rainbow", 0).is_some());
        assert!(create("fire", 42).is_some());
    }

    #[test]
    fn create_unknown_name_fails() {
        assert!(create("not-a-real-animation", 0).is_none());
    }

    #[test]
    fn every_animation_produces_a_frame_without_panicking() {
        use crate::animation::{AnimationContext, AnimationParams};
        let ctx = AnimationContext {
            params: AnimationParams::default(),
        };
        for name in names() {
            let mut anim = create(name, 1).unwrap();
            for i in 0..120 {
                let t = i as f64 * (1.0 / 30.0);
                let _ = anim.update(t, 1.0 / 30.0, &ctx);
            }
            anim.reset();
            let _ = anim.update(0.0, 0.0, &ctx);
        }
    }
}
