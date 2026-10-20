//! Named color palettes (spec sections 62-64): a handful of built-in
//! palettes, plus user-defined ones loaded from
//! `$XDG_CONFIG_HOME/kbdanim/palettes/<name>.toml`, each containing a
//! `colors = ["#rrggbb", ...]` array.

use crate::color::Color;
use crate::toml_lite;
use std::path::Path;

pub fn builtin_names() -> &'static [&'static str] {
    &[
        "warm",
        "cool",
        "fire",
        "ocean",
        "forest",
        "sunset",
        "cyberpunk",
        "pastel",
        "monochrome",
        "neon",
    ]
}

pub fn builtin_palette(name: &str) -> Option<Vec<Color>> {
    let colors: &[(u8, u8, u8)] = match name {
        "warm" => &[(255, 140, 20), (255, 90, 0), (255, 190, 60)],
        "cool" => &[(0, 140, 255), (0, 200, 255), (0, 255, 220)],
        "fire" => &[(255, 0, 0), (255, 120, 0), (255, 220, 0)],
        "ocean" => &[(0, 80, 160), (0, 150, 200), (0, 220, 220)],
        "forest" => &[(20, 90, 40), (60, 140, 60), (120, 180, 80)],
        "sunset" => &[(255, 176, 59), (255, 111, 97), (219, 74, 122), (123, 63, 148)],
        "cyberpunk" => &[(255, 0, 180), (0, 255, 255), (120, 0, 255)],
        "pastel" => &[(255, 209, 220), (209, 255, 220), (209, 220, 255), (255, 255, 209)],
        "monochrome" => &[(255, 255, 255), (180, 180, 180), (90, 90, 90), (0, 0, 0)],
        "neon" => &[(255, 0, 255), (0, 255, 255), (255, 255, 0), (0, 255, 0)],
        _ => return None,
    };
    Some(colors.iter().map(|(r, g, b)| Color::new(*r, *g, *b)).collect())
}

pub fn load_user_palette(dir: &Path, name: &str) -> Option<Vec<Color>> {
    let path = dir.join(format!("{name}.toml"));
    let text = std::fs::read_to_string(path).ok()?;
    let doc = toml_lite::parse(&text).ok()?;
    let strings = doc.root.get_str_array("colors")?;
    let mut out = Vec::with_capacity(strings.len());
    for s in strings {
        out.push(Color::parse(&s).ok()?);
    }
    if out.len() < 2 {
        None
    } else {
        Some(out)
    }
}

/// User-defined palettes take priority over built-in ones of the same
/// name.
pub fn resolve(dir: &Path, name: &str) -> Option<Vec<Color>> {
    load_user_palette(dir, name).or_else(|| builtin_palette(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_builtin_names_resolve() {
        for name in builtin_names() {
            assert!(builtin_palette(name).is_some(), "missing palette: {name}");
        }
    }

    #[test]
    fn builtin_palettes_have_at_least_two_colors() {
        for name in builtin_names() {
            let p = builtin_palette(name).unwrap();
            assert!(p.len() >= 2);
        }
    }

    #[test]
    fn unknown_palette_is_none() {
        assert!(builtin_palette("not-a-real-palette").is_none());
    }
}
