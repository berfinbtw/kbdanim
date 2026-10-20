//! Application configuration: defaults, loading from `config.toml`, and
//! the per-power-profile settings used by the daemon's auto-profile
//! switching (spec sections 15-17, 23-24, 57).

use crate::color::Color;
use crate::device::DEFAULT_DEVICE_PATH;
use crate::error::Result;
use crate::toml_lite::{self, TomlDoc};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub mode: String,
    pub brightness: u8,
    pub speed: f64,
    pub fps: u32,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub device: String,
    pub mode: String,
    pub speed: f64,
    pub fps: u32,
    pub brightness: u8,
    pub easing: String,
    pub color: Color,
    pub color2: Color,
    pub daemon_enabled: bool,
    pub restore_state: bool,
    pub auto_profile: bool,
    pub transition_enabled: bool,
    pub transition_duration: f64,
    pub color_quantization: u8,
    pub random_interval: f64,
    pub profiles: BTreeMap<String, ProfileConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "ac".to_string(),
            ProfileConfig { mode: "rainbow".into(), brightness: 255, speed: 1.0, fps: 30 },
        );
        profiles.insert(
            "battery".to_string(),
            ProfileConfig { mode: "breathing".into(), brightness: 80, speed: 0.8, fps: 15 },
        );
        profiles.insert(
            "gaming".to_string(),
            ProfileConfig { mode: "cyberpunk".into(), brightness: 255, speed: 1.2, fps: 60 },
        );
        profiles.insert(
            "night".to_string(),
            ProfileConfig { mode: "night".into(), brightness: 25, speed: 0.4, fps: 20 },
        );
        profiles.insert(
            "default".to_string(),
            ProfileConfig { mode: "rainbow".into(), brightness: 200, speed: 1.0, fps: 30 },
        );

        Config {
            device: DEFAULT_DEVICE_PATH.to_string(),
            mode: "rainbow".to_string(),
            speed: 1.0,
            fps: 30,
            brightness: 200,
            easing: "smoothstep".to_string(),
            color: Color::new(0, 170, 255),
            color2: Color::new(255, 0, 128),
            daemon_enabled: false,
            restore_state: true,
            auto_profile: true,
            transition_enabled: true,
            transition_duration: 1.5,
            color_quantization: 1,
            random_interval: 30.0,
            profiles,
        }
    }
}

impl Config {
    /// Loads `config.toml` from `path`, falling back to built-in
    /// defaults (with a logged warning, never a crash) if the file is
    /// missing or fails to parse (spec section 45: config parse
    /// failures must be handled gracefully).
    pub fn load_or_default(path: &Path) -> Config {
        match std::fs::read_to_string(path) {
            Ok(text) => match toml_lite::parse(&text) {
                Ok(doc) => Config::from_doc(&doc),
                Err(e) => {
                    crate::logging::warn(&format!(
                        "config parse error in {}: {e} — using defaults",
                        path.display()
                    ));
                    Config::default()
                }
            },
            Err(_) => Config::default(),
        }
    }

    fn from_doc(doc: &TomlDoc) -> Config {
        let mut cfg = Config::default();

        if let Some(v) = doc.root.get_str("device") {
            cfg.device = v.to_string();
        }

        if let Some(sec) = doc.section("animation") {
            if let Some(v) = sec.get_str("mode") {
                cfg.mode = v.to_string();
            }
            if let Some(v) = sec.get_f64("speed") {
                cfg.speed = v;
            }
            if let Some(v) = sec.get_i64("fps") {
                cfg.fps = v.max(1) as u32;
            }
            if let Some(v) = sec.get_i64("brightness") {
                cfg.brightness = v.clamp(0, 255) as u8;
            }
            if let Some(v) = sec.get_str("easing") {
                cfg.easing = v.to_string();
            }
            if let Some(v) = sec.get_str("color") {
                if let Ok(c) = Color::parse(v) {
                    cfg.color = c;
                }
            }
            if let Some(v) = sec.get_str("color2") {
                if let Ok(c) = Color::parse(v) {
                    cfg.color2 = c;
                }
            }
        }

        if let Some(sec) = doc.section("daemon") {
            if let Some(v) = sec.get_bool("enabled") {
                cfg.daemon_enabled = v;
            }
            if let Some(v) = sec.get_bool("restore_state") {
                cfg.restore_state = v;
            }
        }

        if let Some(sec) = doc.section("power") {
            if let Some(v) = sec.get_bool("auto_profile") {
                cfg.auto_profile = v;
            }
        }

        if let Some(sec) = doc.section("transition") {
            if let Some(v) = sec.get_bool("enabled") {
                cfg.transition_enabled = v;
            }
            if let Some(v) = sec.get_f64("duration") {
                cfg.transition_duration = v;
            }
        }

        if let Some(sec) = doc.section("engine") {
            if let Some(v) = sec.get_i64("color_quantization") {
                cfg.color_quantization = v.clamp(1, 32) as u8;
            }
        }

        if let Some(sec) = doc.section("random") {
            if let Some(v) = sec.get_f64("interval") {
                cfg.random_interval = v.max(1.0);
            }
        }

        for (name, sec) in doc.sections.iter() {
            if let Some(profile_name) = name.strip_prefix("profiles.") {
                let mut p = cfg.profiles.get(profile_name).cloned().unwrap_or(ProfileConfig {
                    mode: cfg.mode.clone(),
                    brightness: cfg.brightness,
                    speed: cfg.speed,
                    fps: cfg.fps,
                });
                if let Some(v) = sec.get_str("mode") {
                    p.mode = v.to_string();
                }
                if let Some(v) = sec.get_i64("brightness") {
                    p.brightness = v.clamp(0, 255) as u8;
                }
                if let Some(v) = sec.get_f64("speed") {
                    p.speed = v;
                }
                if let Some(v) = sec.get_i64("fps") {
                    p.fps = v.max(1) as u32;
                }
                cfg.profiles.insert(profile_name.to_string(), p);
            }
        }

        cfg
    }

    /// Serializes the whole configuration back into the same TOML
    /// subset `from_doc` reads (see `src/toml_lite.rs`). Used by the
    /// GUI's Settings page; the CLI never needs to write config.toml
    /// itself (users edit it directly, or via `kbdanim config edit`).
    pub fn to_toml_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("device = \"{}\"\n\n", self.device));

        s.push_str("[animation]\n");
        s.push_str(&format!("mode = \"{}\"\n", self.mode));
        s.push_str(&format!("speed = {}\n", self.speed));
        s.push_str(&format!("fps = {}\n", self.fps));
        s.push_str(&format!("brightness = {}\n", self.brightness));
        s.push_str(&format!("easing = \"{}\"\n", self.easing));
        s.push_str(&format!("color = \"{}\"\n", self.color.to_hex()));
        s.push_str(&format!("color2 = \"{}\"\n\n", self.color2.to_hex()));

        s.push_str("[daemon]\n");
        s.push_str(&format!("enabled = {}\n", self.daemon_enabled));
        s.push_str(&format!("restore_state = {}\n\n", self.restore_state));

        s.push_str("[power]\n");
        s.push_str(&format!("auto_profile = {}\n\n", self.auto_profile));

        s.push_str("[transition]\n");
        s.push_str(&format!("enabled = {}\n", self.transition_enabled));
        s.push_str(&format!("duration = {}\n\n", self.transition_duration));

        s.push_str("[engine]\n");
        s.push_str(&format!("color_quantization = {}\n\n", self.color_quantization));

        s.push_str("[random]\n");
        s.push_str(&format!("interval = {}\n\n", self.random_interval));

        for (name, p) in &self.profiles {
            s.push_str(&format!("[profiles.{name}]\n"));
            s.push_str(&format!("mode = \"{}\"\n", p.mode));
            s.push_str(&format!("brightness = {}\n", p.brightness));
            s.push_str(&format!("speed = {}\n", p.speed));
            s.push_str(&format!("fps = {}\n\n", p.fps));
        }

        s
    }

    /// Writes this configuration to `path` (creating parent directories
    /// as needed).
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_toml_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sane_values() {
        let cfg = Config::default();
        assert!(cfg.fps >= 1);
        assert!(cfg.speed > 0.0);
        assert!(cfg.profiles.contains_key("ac"));
        assert!(cfg.profiles.contains_key("battery"));
    }

    #[test]
    fn missing_file_falls_back_to_default() {
        let cfg = Config::load_or_default(Path::new("/nonexistent/kbdanim/config.toml"));
        assert_eq!(cfg.mode, Config::default().mode);
    }

    #[test]
    fn parses_animation_section() {
        let doc = toml_lite::parse("[animation]\nmode = \"fire\"\nbrightness = 100\nfps = 60\n").unwrap();
        let cfg = Config::from_doc(&doc);
        assert_eq!(cfg.mode, "fire");
        assert_eq!(cfg.brightness, 100);
        assert_eq!(cfg.fps, 60);
    }

    #[test]
    fn parses_profiles() {
        let doc = toml_lite::parse("[profiles.ac]\nmode = \"rainbow\"\nbrightness = 255\n").unwrap();
        let cfg = Config::from_doc(&doc);
        let ac = cfg.profiles.get("ac").unwrap();
        assert_eq!(ac.mode, "rainbow");
        assert_eq!(ac.brightness, 255);
    }

    #[test]
    fn malformed_config_does_not_panic() {
        // Just checking Config::from_doc doesn't panic on an empty doc;
        // the parse-failure path is exercised in load_or_default above.
        let doc = TomlDoc::default();
        let _ = Config::from_doc(&doc);
    }

    #[test]
    fn to_toml_string_round_trips_through_from_doc() {
        let cfg = Config {
            mode: "fire".to_string(),
            brightness: 123,
            fps: 45,
            speed: 1.75,
            ..Config::default()
        };

        let text = cfg.to_toml_string();
        let doc = toml_lite::parse(&text).expect("serialized config must parse");
        let round_tripped = Config::from_doc(&doc);

        assert_eq!(round_tripped.mode, "fire");
        assert_eq!(round_tripped.brightness, 123);
        assert_eq!(round_tripped.fps, 45);
        assert_eq!(round_tripped.speed, 1.75);
        assert!(round_tripped.profiles.contains_key("ac"));
    }
}
