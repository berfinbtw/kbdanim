//! A deliberately minimal TOML-subset parser.
//!
//! kbdanim avoids the `toml`/`serde` crates (see Cargo.toml). This
//! module supports exactly what kbdanim's own config, preset, palette,
//! playlist, and state files need:
//!
//! - `# comments` (to end of line, quote-aware)
//! - top-level `key = value` pairs
//! - `[section]` and `[section.subsection]` headers
//! - `[[array.of.tables]]` headers (one level, used by playlists)
//! - string (`"..."`), integer, float, and boolean values
//! - arrays of strings (`["a", "b"]`), used by palettes
//!
//! It is not a general-purpose TOML implementation and does not need to
//! be, since kbdanim only ever reads files in this exact shape.

use crate::error::{KbdAnimError, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TomlValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<TomlValue>),
}

impl TomlValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            TomlValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            TomlValue::Integer(i) => Some(*i),
            TomlValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            TomlValue::Float(f) => Some(*f),
            TomlValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            TomlValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_str_array(&self) -> Option<Vec<String>> {
        match self {
            TomlValue::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(item.as_str()?.to_string());
                }
                Some(out)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TomlSection {
    pub values: BTreeMap<String, TomlValue>,
}

impl TomlSection {
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(|v| v.as_str())
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.values.get(key).and_then(|v| v.as_i64())
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.values.get(key).and_then(|v| v.as_f64())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key).and_then(|v| v.as_bool())
    }

    pub fn get_str_array(&self, key: &str) -> Option<Vec<String>> {
        self.values.get(key).and_then(|v| v.as_str_array())
    }
}

#[derive(Debug, Clone, Default)]
pub struct TomlDoc {
    pub root: TomlSection,
    pub sections: BTreeMap<String, TomlSection>,
    pub array_sections: BTreeMap<String, Vec<TomlSection>>,
}

impl TomlDoc {
    pub fn section(&self, name: &str) -> Option<&TomlSection> {
        self.sections.get(name)
    }

    pub fn array(&self, name: &str) -> Option<&Vec<TomlSection>> {
        self.array_sections.get(name)
    }
}

enum Current {
    Root,
    Section(String),
    Array(String),
}

pub fn parse(input: &str) -> Result<TomlDoc> {
    let mut doc = TomlDoc::default();
    let mut current = Current::Root;

    for (i, raw_line) in input.lines().enumerate() {
        let lineno = i + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        if let Some(name) = parse_array_header(line) {
            doc.array_sections
                .entry(name.clone())
                .or_default()
                .push(TomlSection::default());
            current = Current::Array(name);
            continue;
        }

        if let Some(name) = parse_section_header(line) {
            doc.sections.entry(name.clone()).or_default();
            current = Current::Section(name);
            continue;
        }

        let (key, raw_value) = split_key_value(line, lineno)?;
        let value = parse_value(&raw_value, lineno)?;

        match &current {
            Current::Root => {
                doc.root.values.insert(key, value);
            }
            Current::Section(name) => {
                doc.sections.entry(name.clone()).or_default().values.insert(key, value);
            }
            Current::Array(name) => {
                if let Some(last) = doc.array_sections.get_mut(name).and_then(|v| v.last_mut()) {
                    last.values.insert(key, value);
                }
            }
        }
    }

    Ok(doc)
}

fn parse_array_header(line: &str) -> Option<String> {
    if line.len() > 4 && line.starts_with("[[") && line.ends_with("]]") {
        Some(line[2..line.len() - 2].trim().to_string())
    } else {
        None
    }
}

fn parse_section_header(line: &str) -> Option<String> {
    if line.len() > 2 && line.starts_with('[') && line.ends_with(']') && !line.starts_with("[[") {
        Some(line[1..line.len() - 1].trim().to_string())
    } else {
        None
    }
}

/// Strips a `#` comment, but only outside of a quoted string.
fn strip_comment(line: &str) -> &str {
    let mut in_quotes = false;
    for (idx, ch) in line.char_indices() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '#' if !in_quotes => return &line[..idx],
            _ => {}
        }
    }
    line
}

fn split_key_value(line: &str, lineno: usize) -> Result<(String, String)> {
    match line.find('=') {
        Some(idx) => {
            let key = line[..idx].trim().trim_matches('"').to_string();
            let val = line[idx + 1..].trim().to_string();
            if key.is_empty() {
                return Err(KbdAnimError::ConfigParse(format!(
                    "line {lineno}: missing key before '='"
                )));
            }
            Ok((key, val))
        }
        None => Err(KbdAnimError::ConfigParse(format!(
            "line {lineno}: expected 'key = value', got: {line}"
        ))),
    }
}

fn parse_value(raw: &str, lineno: usize) -> Result<TomlValue> {
    let raw = raw.trim();
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        return Ok(TomlValue::String(raw[1..raw.len() - 1].to_string()));
    }
    if raw == "true" {
        return Ok(TomlValue::Bool(true));
    }
    if raw == "false" {
        return Ok(TomlValue::Bool(false));
    }
    if raw.starts_with('[') && raw.ends_with(']') {
        let inner = &raw[1..raw.len() - 1];
        let mut items = Vec::new();
        for part in split_top_level_commas(inner) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            items.push(parse_value(part, lineno)?);
        }
        return Ok(TomlValue::Array(items));
    }
    if let Ok(i) = raw.parse::<i64>() {
        return Ok(TomlValue::Integer(i));
    }
    if let Ok(f) = raw.parse::<f64>() {
        return Ok(TomlValue::Float(f));
    }
    Err(KbdAnimError::ConfigParse(format!(
        "line {lineno}: could not parse value: {raw}"
    )))
}

fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut depth = 0i32;
    let mut in_quotes = false;
    let mut current = String::new();
    for ch in s.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            '[' if !in_quotes => {
                depth += 1;
                current.push(ch);
            }
            ']' if !in_quotes => {
                depth -= 1;
                current.push(ch);
            }
            ',' if !in_quotes && depth == 0 => {
                result.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        result.push(current);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_root_keys() {
        let doc = parse("device = \"/sys/class/leds/x\"\nfps = 30\nspeed = 1.5\nenabled = true").unwrap();
        assert_eq!(doc.root.get_str("device"), Some("/sys/class/leds/x"));
        assert_eq!(doc.root.get_i64("fps"), Some(30));
        assert_eq!(doc.root.get_f64("speed"), Some(1.5));
        assert_eq!(doc.root.get_bool("enabled"), Some(true));
    }

    #[test]
    fn parses_sections() {
        let text = "[animation]\nmode = \"rainbow\"\nbrightness = 255\n\n[daemon]\nenabled = false\n";
        let doc = parse(text).unwrap();
        assert_eq!(doc.section("animation").unwrap().get_str("mode"), Some("rainbow"));
        assert_eq!(doc.section("animation").unwrap().get_i64("brightness"), Some(255));
        assert_eq!(doc.section("daemon").unwrap().get_bool("enabled"), Some(false));
    }

    #[test]
    fn parses_dotted_sections() {
        let text = "[profiles.ac]\nmode = \"rainbow\"\n[profiles.battery]\nmode = \"breathing\"\n";
        let doc = parse(text).unwrap();
        assert_eq!(doc.section("profiles.ac").unwrap().get_str("mode"), Some("rainbow"));
        assert_eq!(
            doc.section("profiles.battery").unwrap().get_str("mode"),
            Some("breathing")
        );
    }

    #[test]
    fn parses_string_array() {
        let doc = parse("colors = [\"#ff0000\", \"#00ff00\", \"#0000ff\"]").unwrap();
        assert_eq!(
            doc.root.get_str_array("colors"),
            Some(vec![
                "#ff0000".to_string(),
                "#00ff00".to_string(),
                "#0000ff".to_string()
            ])
        );
    }

    #[test]
    fn parses_array_of_tables() {
        let text = "[[animations]]\nmode = \"rainbow\"\nduration = 30\n\n[[animations]]\nmode = \"fire\"\nduration = 20\n";
        let doc = parse(text).unwrap();
        let arr = doc.array("animations").unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].get_str("mode"), Some("rainbow"));
        assert_eq!(arr[0].get_i64("duration"), Some(30));
        assert_eq!(arr[1].get_str("mode"), Some("fire"));
    }

    #[test]
    fn ignores_comments() {
        let doc = parse("# a comment\nfps = 30 # inline comment\n").unwrap();
        assert_eq!(doc.root.get_i64("fps"), Some(30));
    }

    #[test]
    fn hash_inside_string_is_not_a_comment() {
        let doc = parse("name = \"gaming #1\"\n").unwrap();
        assert_eq!(doc.root.get_str("name"), Some("gaming #1"));
    }

    #[test]
    fn malformed_line_is_a_parse_error_not_a_panic() {
        assert!(parse("this is not valid toml").is_err());
    }

    #[test]
    fn blank_lines_and_whitespace_are_ignored() {
        let doc = parse("\n\n  fps = 10  \n\n").unwrap();
        assert_eq!(doc.root.get_i64("fps"), Some(10));
    }
}
