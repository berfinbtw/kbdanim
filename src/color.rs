//! Color engine: RGB storage type plus HSV/HSL representations,
//! conversions between them, named-color and hex/rgb-triplet parsing,
//! and interpolation helpers used by the animation engine.

use crate::error::{KbdAnimError, Result};
use crate::rng::Rng;

/// An 8-bit-per-channel RGB color, the canonical representation written
/// to the keyboard's `multi_intensity` sysfs file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);

    /// Parses a color from a name (`"red"`), a hex code (`"#ff0000"`),
    /// or a comma/space separated RGB triplet (`"255,0,0"` / `"255 0 0"`).
    pub fn parse(s: &str) -> Result<Color> {
        let s = s.trim();
        if s.is_empty() {
            return Err(KbdAnimError::InvalidColor("empty color string".to_string()));
        }
        if let Some(hex) = s.strip_prefix('#') {
            return Color::from_hex(hex);
        }
        if let Some(c) = named_color(s) {
            return Ok(c);
        }
        if let Some(c) = Color::from_triplet(s) {
            return Ok(c);
        }
        Err(KbdAnimError::InvalidColor(format!(
            "'{s}' is not a valid color (expected a name like 'red', a hex code like '#ff8800', or 'r,g,b')"
        )))
    }

    /// Parses a `#rrggbb` or `#rgb` hex string (without the leading `#`).
    pub fn from_hex(hex: &str) -> Result<Color> {
        let hex = hex.trim();
        if hex.is_empty() || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(KbdAnimError::InvalidColor(format!("invalid hex color: #{hex}")));
        }
        let full: String = match hex.chars().count() {
            3 => {
                let mut s = String::with_capacity(6);
                for c in hex.chars() {
                    s.push(c);
                    s.push(c);
                }
                s
            }
            6 => hex.to_string(),
            _ => {
                return Err(KbdAnimError::InvalidColor(format!(
                    "hex color must have 3 or 6 digits: #{hex}"
                )))
            }
        };
        let byte = |part: &str| -> Result<u8> {
            u8::from_str_radix(part, 16)
                .map_err(|_| KbdAnimError::InvalidColor(format!("invalid hex digits: #{hex}")))
        };
        let r = byte(&full[0..2])?;
        let g = byte(&full[2..4])?;
        let b = byte(&full[4..6])?;
        Ok(Color::new(r, g, b))
    }

    fn from_triplet(s: &str) -> Option<Color> {
        let parts: Vec<&str> = if s.contains(',') {
            s.split(',').map(|p| p.trim()).collect()
        } else {
            s.split_whitespace().collect()
        };
        if parts.len() != 3 {
            return None;
        }
        let mut vals = [0u8; 3];
        for (i, p) in parts.iter().enumerate() {
            let v: u16 = p.parse().ok()?;
            if v > 255 {
                return None;
            }
            vals[i] = v as u8;
        }
        Some(Color::new(vals[0], vals[1], vals[2]))
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn to_hsv(&self) -> Hsv {
        let rn = self.r as f64 / 255.0;
        let gn = self.g as f64 / 255.0;
        let bn = self.b as f64 / 255.0;
        let max = rn.max(gn).max(bn);
        let min = rn.min(gn).min(bn);
        let delta = max - min;
        let h = hue_from_rgb(rn, gn, bn, max, delta);
        let s = if max <= 1e-12 { 0.0 } else { delta / max };
        Hsv { h, s, v: max }
    }

    pub fn to_hsl(&self) -> Hsl {
        let rn = self.r as f64 / 255.0;
        let gn = self.g as f64 / 255.0;
        let bn = self.b as f64 / 255.0;
        let max = rn.max(gn).max(bn);
        let min = rn.min(gn).min(bn);
        let delta = max - min;
        let h = hue_from_rgb(rn, gn, bn, max, delta);
        let l = (max + min) / 2.0;
        let s = if delta.abs() < 1e-12 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };
        Hsl { h, s, l }
    }

    /// Linearly interpolates each RGB channel independently. `t` is
    /// clamped to [0, 1].
    pub fn lerp(a: Color, b: Color, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        let ch = |x: u8, y: u8| -> u8 {
            (x as f64 + (y as f64 - x as f64) * t)
                .round()
                .clamp(0.0, 255.0) as u8
        };
        Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b))
    }

    /// A fully saturated, full-value random color (random hue).
    pub fn random_vivid(rng: &mut Rng) -> Color {
        let hue = rng.range_f64(0.0, 360.0);
        Hsv { h: hue, s: 1.0, v: 1.0 }.to_rgb()
    }

    /// A soft, desaturated random color in the "pastel" family.
    pub fn random_pastel(rng: &mut Rng) -> Color {
        let hue = rng.range_f64(0.0, 360.0);
        let sat = rng.range_f64(0.28, 0.5);
        let val = rng.range_f64(0.85, 1.0);
        Hsv { h: hue, s: sat, v: val }.to_rgb()
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Hue/Saturation/Value color, `h` in degrees [0, 360), `s`/`v` in [0, 1].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub h: f64,
    pub s: f64,
    pub v: f64,
}

impl Hsv {
    pub fn new(h: f64, s: f64, v: f64) -> Self {
        Hsv {
            h: h.rem_euclid(360.0),
            s: s.clamp(0.0, 1.0),
            v: v.clamp(0.0, 1.0),
        }
    }

    pub fn to_rgb(&self) -> Color {
        let h = self.h.rem_euclid(360.0);
        let s = self.s.clamp(0.0, 1.0);
        let v = self.v.clamp(0.0, 1.0);
        let c = v * s;
        let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
        let m = v - c;
        let (r1, g1, b1) = hue_bucket(h, c, x);
        let to_byte = |val: f64| -> u8 { ((val + m) * 255.0).round().clamp(0.0, 255.0) as u8 };
        Color::new(to_byte(r1), to_byte(g1), to_byte(b1))
    }
}

/// Hue/Saturation/Lightness color, `h` in degrees [0, 360), `s`/`l` in [0, 1].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub h: f64,
    pub s: f64,
    pub l: f64,
}

impl Hsl {
    pub fn new(h: f64, s: f64, l: f64) -> Self {
        Hsl {
            h: h.rem_euclid(360.0),
            s: s.clamp(0.0, 1.0),
            l: l.clamp(0.0, 1.0),
        }
    }

    pub fn to_rgb(&self) -> Color {
        let h = self.h.rem_euclid(360.0);
        let s = self.s.clamp(0.0, 1.0);
        let l = self.l.clamp(0.0, 1.0);
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
        let m = l - c / 2.0;
        let (r1, g1, b1) = hue_bucket(h, c, x);
        let to_byte = |val: f64| -> u8 { ((val + m) * 255.0).round().clamp(0.0, 255.0) as u8 };
        Color::new(to_byte(r1), to_byte(g1), to_byte(b1))
    }
}

/// Shared hue-to-RGB-bucket logic used by both HSV->RGB and HSL->RGB
/// (they differ only in how `c`/`x`/`m` are derived, not in this step).
fn hue_bucket(h: f64, c: f64, x: f64) -> (f64, f64, f64) {
    if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    }
}

/// The hue (in degrees) of a normalized (0..1) RGB triplet, given its
/// max channel value and max-min delta. Shared by RGB->HSV and RGB->HSL.
fn hue_from_rgb(rn: f64, gn: f64, bn: f64, max: f64, delta: f64) -> f64 {
    if delta.abs() < 1e-12 {
        return 0.0;
    }
    let h = if (max - rn).abs() < 1e-12 {
        60.0 * (((gn - bn) / delta).rem_euclid(6.0))
    } else if (max - gn).abs() < 1e-12 {
        60.0 * (((bn - rn) / delta) + 2.0)
    } else {
        60.0 * (((rn - gn) / delta) + 4.0)
    };
    h.rem_euclid(360.0)
}

/// Interpolates between two hues along the shortest arc of the color
/// wheel (e.g. 350 -> 10 moves forward through 0, not backward through
/// 180). Used by animations that glide between hues.
pub fn hue_lerp(h1: f64, h2: f64, t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    let mut diff = (h2 - h1) % 360.0;
    if diff > 180.0 {
        diff -= 360.0;
    } else if diff < -180.0 {
        diff += 360.0;
    }
    (h1 + diff * t).rem_euclid(360.0)
}

fn named_color(name: &str) -> Option<Color> {
    let key = name.to_lowercase();
    NAMED_COLORS
        .iter()
        .find(|(n, _, _, _)| *n == key)
        .map(|(_, r, g, b)| Color::new(*r, *g, *b))
}

/// A generous set of CSS-like named colors. Not exhaustive, but well
/// beyond the minimum list requested by the spec.
const NAMED_COLORS: &[(&str, u8, u8, u8)] = &[
    ("black", 0, 0, 0),
    ("white", 255, 255, 255),
    ("red", 255, 0, 0),
    ("green", 0, 128, 0),
    ("blue", 0, 0, 255),
    ("yellow", 255, 255, 0),
    ("cyan", 0, 255, 255),
    ("magenta", 255, 0, 255),
    ("orange", 255, 165, 0),
    ("purple", 128, 0, 128),
    ("pink", 255, 192, 203),
    ("gray", 128, 128, 128),
    ("grey", 128, 128, 128),
    ("silver", 192, 192, 192),
    ("gold", 255, 215, 0),
    ("brown", 165, 42, 42),
    ("navy", 0, 0, 128),
    ("teal", 0, 128, 128),
    ("lime", 0, 255, 0),
    ("olive", 128, 128, 0),
    ("maroon", 128, 0, 0),
    ("indigo", 75, 0, 130),
    ("violet", 238, 130, 238),
    ("turquoise", 64, 224, 208),
    ("coral", 255, 127, 80),
    ("salmon", 250, 128, 114),
    ("khaki", 240, 230, 140),
    ("crimson", 220, 20, 60),
    ("chartreuse", 127, 255, 0),
    ("orchid", 218, 112, 214),
    ("plum", 221, 160, 221),
    ("tan", 210, 180, 140),
    ("beige", 245, 245, 220),
    ("ivory", 255, 255, 240),
    ("lavender", 230, 230, 250),
    ("mint", 189, 252, 201),
    ("skyblue", 135, 206, 235),
    ("steelblue", 70, 130, 180),
    ("royalblue", 65, 105, 225),
    ("dodgerblue", 30, 144, 255),
    ("deepskyblue", 0, 191, 255),
    ("aqua", 0, 255, 255),
    ("aquamarine", 127, 255, 212),
    ("forestgreen", 34, 139, 34),
    ("seagreen", 46, 139, 87),
    ("springgreen", 0, 255, 127),
    ("limegreen", 50, 205, 50),
    ("darkgreen", 0, 100, 0),
    ("darkred", 139, 0, 0),
    ("firebrick", 178, 34, 34),
    ("tomato", 255, 99, 71),
    ("orangered", 255, 69, 0),
    ("darkorange", 255, 140, 0),
    ("amber", 255, 191, 0),
    ("hotpink", 255, 105, 180),
    ("deeppink", 255, 20, 147),
    ("fuchsia", 255, 0, 255),
    ("darkviolet", 148, 0, 211),
    ("darkorchid", 153, 50, 204),
    ("mediumpurple", 147, 112, 219),
    ("slateblue", 106, 90, 205),
    ("darkslateblue", 72, 61, 139),
    ("midnightblue", 25, 25, 112),
    ("cornflowerblue", 100, 149, 237),
    ("powderblue", 176, 224, 230),
    ("lightblue", 173, 216, 230),
    ("lightgreen", 144, 238, 144),
    ("lightyellow", 255, 255, 224),
    ("lightpink", 255, 182, 193),
    ("lightgray", 211, 211, 211),
    ("lightgrey", 211, 211, 211),
    ("darkgray", 169, 169, 169),
    ("darkgrey", 169, 169, 169),
    ("dimgray", 105, 105, 105),
    ("dimgrey", 105, 105, 105),
    ("slategray", 112, 128, 144),
    ("slategrey", 112, 128, 144),
    ("charcoal", 54, 69, 79),
    ("wheat", 245, 222, 179),
    ("peru", 205, 133, 63),
    ("chocolate", 210, 105, 30),
    ("sienna", 160, 82, 45),
    ("rosybrown", 188, 143, 143),
    ("thistle", 216, 191, 216),
    ("mediumorchid", 186, 85, 211),
    ("mediumvioletred", 199, 21, 133),
    ("palevioletred", 219, 112, 147),
    ("cadetblue", 95, 158, 160),
    ("darkcyan", 0, 139, 139),
    ("darkturquoise", 0, 206, 209),
    ("mediumturquoise", 72, 209, 204),
    ("paleturquoise", 175, 238, 238),
    ("honeydew", 240, 255, 240),
    ("mintcream", 245, 255, 250),
    ("azure", 240, 255, 255),
    ("snow", 255, 250, 250),
    ("linen", 250, 240, 230),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_six_digit() {
        let c = Color::parse("#ff8800").unwrap();
        assert_eq!(c, Color::new(255, 136, 0));
    }

    #[test]
    fn parse_hex_three_digit() {
        let c = Color::parse("#f80").unwrap();
        assert_eq!(c, Color::new(255, 136, 0));
    }

    #[test]
    fn parse_hex_lowercase_and_uppercase() {
        assert_eq!(Color::parse("#AABBCC").unwrap(), Color::parse("#aabbcc").unwrap());
    }

    #[test]
    fn parse_invalid_hex_is_error() {
        assert!(Color::parse("#zzzzzz").is_err());
        assert!(Color::parse("#12345").is_err());
    }

    #[test]
    fn parse_named_colors() {
        assert_eq!(Color::parse("red").unwrap(), Color::new(255, 0, 0));
        assert_eq!(Color::parse("RED").unwrap(), Color::new(255, 0, 0));
        assert_eq!(Color::parse("blue").unwrap(), Color::new(0, 0, 255));
        assert_eq!(Color::parse("white").unwrap(), Color::new(255, 255, 255));
    }

    #[test]
    fn parse_rgb_triplet_comma() {
        assert_eq!(Color::parse("255,120,20").unwrap(), Color::new(255, 120, 20));
    }

    #[test]
    fn parse_rgb_triplet_space() {
        assert_eq!(Color::parse("255 120 20").unwrap(), Color::new(255, 120, 20));
    }

    #[test]
    fn parse_rgb_triplet_out_of_range() {
        assert!(Color::parse("300,0,0").is_err());
    }

    #[test]
    fn parse_unknown_is_error() {
        assert!(Color::parse("not-a-color").is_err());
    }

    #[test]
    fn hsv_round_trip_primaries() {
        for c in [Color::new(255, 0, 0), Color::new(0, 255, 0), Color::new(0, 0, 255)] {
            let back = c.to_hsv().to_rgb();
            assert_eq!(c, back);
        }
    }

    #[test]
    fn hsl_round_trip_primaries() {
        for c in [Color::new(255, 0, 0), Color::new(0, 255, 0), Color::new(0, 0, 255)] {
            let back = c.to_hsl().to_rgb();
            assert_eq!(c, back);
        }
    }

    #[test]
    fn hsv_round_trip_grayscale() {
        for v in [Color::BLACK, Color::WHITE, Color::new(128, 128, 128)] {
            let back = v.to_hsv().to_rgb();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn hue_zero_is_red() {
        let c = Hsv::new(0.0, 1.0, 1.0).to_rgb();
        assert_eq!(c, Color::new(255, 0, 0));
    }

    #[test]
    fn hue_120_is_green() {
        let c = Hsv::new(120.0, 1.0, 1.0).to_rgb();
        assert_eq!(c, Color::new(0, 255, 0));
    }

    #[test]
    fn hue_240_is_blue() {
        let c = Hsv::new(240.0, 1.0, 1.0).to_rgb();
        assert_eq!(c, Color::new(0, 0, 255));
    }

    #[test]
    fn lerp_endpoints() {
        let a = Color::new(0, 0, 0);
        let b = Color::new(255, 255, 255);
        assert_eq!(Color::lerp(a, b, 0.0), a);
        assert_eq!(Color::lerp(a, b, 1.0), b);
    }

    #[test]
    fn lerp_midpoint() {
        let a = Color::new(0, 0, 0);
        let b = Color::new(255, 255, 255);
        let mid = Color::lerp(a, b, 0.5);
        assert!((mid.r as i32 - 128).abs() <= 1);
    }

    #[test]
    fn hue_lerp_shortest_path_wraps_forward() {
        // From 350 to 10 the shortest path goes forward through 0/360,
        // not backward through 180.
        let mid = hue_lerp(350.0, 10.0, 0.5);
        assert!(mid < 5.0 || mid > 355.0);
    }

    #[test]
    fn hue_lerp_endpoints() {
        assert!((hue_lerp(10.0, 100.0, 0.0) - 10.0).abs() < 1e-9);
        assert!((hue_lerp(10.0, 100.0, 1.0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn to_hex_format() {
        assert_eq!(Color::new(255, 8, 0).to_hex(), "#FF0800");
    }
}
