//! Profiles describe the device's overall behavior (which mode,
//! brightness, speed and FPS to use), distinct from presets, which are a
//! single named animation configuration a user picks explicitly (spec
//! section 16). Profiles come from `Config::profiles` and are mainly
//! switched automatically based on AC/battery power state.

use crate::config::{Config, ProfileConfig};

pub fn get<'a>(config: &'a Config, name: &str) -> Option<&'a ProfileConfig> {
    config.profiles.get(name)
}

pub fn list(config: &Config) -> Vec<String> {
    config.profiles.keys().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_profiles_are_listed() {
        let cfg = Config::default();
        let names = list(&cfg);
        assert!(names.contains(&"ac".to_string()));
        assert!(names.contains(&"battery".to_string()));
    }

    #[test]
    fn unknown_profile_is_none() {
        let cfg = Config::default();
        assert!(get(&cfg, "not-a-real-profile").is_none());
    }
}
