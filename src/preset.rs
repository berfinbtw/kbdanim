//! A preset is a single named, complete animation configuration (spec
//! section 15) — distinct from a profile (spec section 16), which
//! describes the device's overall behavior across contexts. Built-in
//! presets always exist; user presets in `presets_dir()/*.toml` can add
//! new ones or override a built-in preset of the same name.

use crate::color::Color;
use crate::toml_lite;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Preset {
    pub name: String,
    pub mode: String,
    pub brightness: u8,
    pub speed: f64,
    pub fps: u32,
    pub color: Option<Color>,
    pub color2: Option<Color>,
    pub easing: Option<String>,
}

pub fn builtin_presets() -> Vec<Preset> {
    vec![
        Preset {
            name: "gaming".into(),
            mode: "cyberpunk".into(),
            brightness: 255,
            speed: 1.2,
            fps: 60,
            color: None,
            color2: None,
            easing: None,
        },
        Preset {
            name: "chill".into(),
            mode: "breathing".into(),
            brightness: 120,
            speed: 0.6,
            fps: 24,
            color: Some(Color::new(0, 170, 255)),
            color2: None,
            easing: Some("sine".into()),
        },
        Preset {
            name: "cyber".into(),
            mode: "cyberpunk".into(),
            brightness: 200,
            speed: 1.0,
            fps: 30,
            color: None,
            color2: None,
            easing: None,
        },
        Preset {
            name: "night".into(),
            mode: "night".into(),
            brightness: 25,
            speed: 0.4,
            fps: 20,
            color: None,
            color2: None,
            easing: None,
        },
        Preset {
            name: "party".into(),
            mode: "party".into(),
            brightness: 255,
            speed: 1.5,
            fps: 45,
            color: None,
            color2: None,
            easing: None,
        },
        Preset {
            name: "focus".into(),
            mode: "static".into(),
            brightness: 100,
            speed: 1.0,
            fps: 10,
            color: Some(Color::new(0, 120, 255)),
            color2: None,
            easing: None,
        },
    ]
}

pub fn load_user_presets(dir: &Path) -> Vec<Preset> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let doc = match toml_lite::parse(&text) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let name = doc
            .root
            .get_str("name")
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("preset")
                    .to_string()
            });
        let mode = doc.root.get_str("mode").unwrap_or("rainbow").to_string();
        let brightness = doc.root.get_i64("brightness").unwrap_or(200).clamp(0, 255) as u8;
        let speed = doc.root.get_f64("speed").unwrap_or(1.0);
        let fps = doc.root.get_i64("fps").unwrap_or(30).max(1) as u32;
        let color = doc.root.get_str("color").and_then(|s| Color::parse(s).ok());
        let color2 = doc.root.get_str("color2").and_then(|s| Color::parse(s).ok());
        let easing = doc.root.get_str("easing").map(|s| s.to_string());
        out.push(Preset {
            name,
            mode,
            brightness,
            speed,
            fps,
            color,
            color2,
            easing,
        });
    }
    out
}

/// All presets, built-in plus user-defined, with user presets
/// overriding a built-in of the same name.
pub fn all_presets(dir: &Path) -> Vec<Preset> {
    let mut map = std::collections::BTreeMap::new();
    for p in builtin_presets() {
        map.insert(p.name.clone(), p);
    }
    for p in load_user_presets(dir) {
        map.insert(p.name.clone(), p);
    }
    map.into_values().collect()
}

pub fn find_preset(dir: &Path, name: &str) -> Option<Preset> {
    all_presets(dir).into_iter().find(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_presets_are_non_empty() {
        assert!(!builtin_presets().is_empty());
    }

    #[test]
    fn find_builtin_preset_by_name() {
        let dir = Path::new("/nonexistent/kbdanim/presets");
        let p = find_preset(dir, "gaming").unwrap();
        assert_eq!(p.mode, "cyberpunk");
    }

    #[test]
    fn unknown_preset_is_none() {
        let dir = Path::new("/nonexistent/kbdanim/presets");
        assert!(find_preset(dir, "not-a-real-preset").is_none());
    }

    #[test]
    fn user_preset_file_is_loaded() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("kbdanim-preset-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("custom.toml");
        std::fs::write(
            &file_path,
            "name = \"custom\"\nmode = \"aurora\"\nbrightness = 150\nspeed = 0.9\nfps = 25\n",
        )
        .unwrap();

        let presets = load_user_presets(&dir);
        assert!(presets.iter().any(|p| p.name == "custom" && p.mode == "aurora"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
