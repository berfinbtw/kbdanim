//! Hand-written CLI argument parsing (no external crate — see
//! Cargo.toml). This module only *parses*; `main.rs` owns dispatch and
//! side effects.

use crate::animation::{default_colors_for, AnimationParams};
use crate::color::Color;
use crate::easing::Easing;
use crate::error::{KbdAnimError, Result};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Every flag kbdanim understands, collected into one bag. Subcommands
/// that don't use a given flag simply ignore that field.
#[derive(Debug, Default, Clone)]
pub struct FlagBag {
    pub mode: Option<String>,
    pub device: Option<String>,
    pub speed: Option<f64>,
    pub fps: Option<u32>,
    pub brightness: Option<u8>,
    pub color: Option<Color>,
    pub color2: Option<Color>,
    pub easing: Option<Easing>,
    pub period: Option<f64>,
    pub randomness: Option<f64>,
    pub saturation: Option<f64>,
    pub temperature: Option<f64>,
    pub intensity: Option<f64>,
    pub palette: Option<String>,
    pub seed: Option<u64>,
    /// Anything that wasn't recognized as a flag (subcommand-specific
    /// positional arguments, e.g. the mode name in `kbdanim mode fire`).
    pub positionals: Vec<String>,
}

fn next_value(args: &[String], i: &mut usize, flag: &str) -> Result<String> {
    *i += 1;
    match args.get(*i) {
        Some(v) => Ok(v.clone()),
        None => Err(KbdAnimError::Other(format!("missing value for {flag}"))),
    }
}

fn parse_speed_flag(value: &str) -> Result<f64> {
    let v: f64 = value
        .parse()
        .map_err(|_| KbdAnimError::InvalidSpeed(value.to_string()))?;
    if !(0.1..=10.0).contains(&v) {
        return Err(KbdAnimError::InvalidSpeed(format!("{value} (must be 0.1-10.0)")));
    }
    Ok(v)
}

fn parse_fps_flag(value: &str) -> Result<u32> {
    let v: u32 = value
        .parse()
        .map_err(|_| KbdAnimError::InvalidFps(value.to_string()))?;
    if !(1..=240).contains(&v) {
        return Err(KbdAnimError::InvalidFps(format!("{value} (must be 1-240)")));
    }
    Ok(v)
}

fn parse_brightness_flag(value: &str) -> Result<u8> {
    let v: i32 = value
        .parse()
        .map_err(|_| KbdAnimError::InvalidBrightness(value.to_string()))?;
    if !(0..=255).contains(&v) {
        return Err(KbdAnimError::InvalidBrightness(format!("{value} (must be 0-255)")));
    }
    Ok(v as u8)
}

fn parse_unit_float_flag(flag: &str, value: &str) -> Result<f64> {
    value
        .parse::<f64>()
        .map_err(|_| KbdAnimError::Other(format!("invalid value for {flag}: {value}")))
}

/// Parses a flat list of `--flag value` pairs (plus bare positionals)
/// into a [`FlagBag`]. Order-independent; unknown `--flags` are a hard
/// error, but bare words are collected as positionals so subcommands
/// like `kbdanim preset run gaming` still work.
pub fn parse_flags(args: &[String]) -> Result<FlagBag> {
    let mut bag = FlagBag::default();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].clone();
        match arg.as_str() {
            "--mode" | "-m" => bag.mode = Some(next_value(args, &mut i, &arg)?),
            "--device" => bag.device = Some(next_value(args, &mut i, &arg)?),
            "--speed" => bag.speed = Some(parse_speed_flag(&next_value(args, &mut i, &arg)?)?),
            "--fps" => bag.fps = Some(parse_fps_flag(&next_value(args, &mut i, &arg)?)?),
            "--brightness" => {
                bag.brightness = Some(parse_brightness_flag(&next_value(args, &mut i, &arg)?)?)
            }
            "--color" => {
                let v = next_value(args, &mut i, &arg)?;
                bag.color = Some(Color::parse(&v)?);
            }
            "--color2" => {
                let v = next_value(args, &mut i, &arg)?;
                bag.color2 = Some(Color::parse(&v)?);
            }
            "--easing" => {
                let v = next_value(args, &mut i, &arg)?;
                bag.easing = Some(
                    Easing::parse(&v)
                        .ok_or_else(|| KbdAnimError::Other(format!("unknown easing: {v}")))?,
                );
            }
            "--period" => {
                bag.period = Some(parse_unit_float_flag("--period", &next_value(args, &mut i, &arg)?)?)
            }
            "--randomness" => {
                bag.randomness =
                    Some(parse_unit_float_flag("--randomness", &next_value(args, &mut i, &arg)?)?)
            }
            "--saturation" => {
                bag.saturation =
                    Some(parse_unit_float_flag("--saturation", &next_value(args, &mut i, &arg)?)?)
            }
            "--temperature" => {
                bag.temperature =
                    Some(parse_unit_float_flag("--temperature", &next_value(args, &mut i, &arg)?)?)
            }
            "--intensity" => {
                bag.intensity =
                    Some(parse_unit_float_flag("--intensity", &next_value(args, &mut i, &arg)?)?)
            }
            "--palette" => bag.palette = Some(next_value(args, &mut i, &arg)?),
            "--seed" => {
                let v = next_value(args, &mut i, &arg)?;
                bag.seed = Some(
                    v.parse::<u64>()
                        .map_err(|_| KbdAnimError::Other(format!("invalid --seed value: {v}")))?,
                );
            }
            other if other.starts_with("--") => {
                return Err(KbdAnimError::Other(format!("unknown flag: {other}")));
            }
            _ => bag.positionals.push(arg.clone()),
        }
        i += 1;
    }
    Ok(bag)
}

/// The fully resolved settings for one animation run: which mode, which
/// device, and the animation parameters to use, with sensible defaults
/// filled in and per-mode default colors (spec: `default_colors_for`)
/// applied wherever the user didn't pass `--color`/`--color2`.
pub struct ResolvedAnimationConfig {
    pub mode: String,
    pub device: String,
    pub fps: u32,
    pub brightness: u8,
    pub params: AnimationParams,
}

pub fn resolve_animation_config(bag: &FlagBag, default_device: &str) -> ResolvedAnimationConfig {
    let mode = bag.mode.clone().unwrap_or_else(|| "rainbow".to_string());
    let (default_color, default_color2) = default_colors_for(&mode);

    let params = AnimationParams {
        color: bag.color.unwrap_or(default_color),
        color2: bag.color2.unwrap_or(default_color2),
        speed: bag.speed.unwrap_or(1.0),
        easing: bag.easing.unwrap_or_default(),
        period: bag.period.unwrap_or(2.0),
        randomness: bag.randomness.unwrap_or(0.5),
        saturation: bag.saturation.unwrap_or(1.0),
        temperature: bag.temperature.unwrap_or(0.5),
        intensity: bag.intensity.unwrap_or(1.0),
        seed: bag.seed.unwrap_or(0),
        ..AnimationParams::default()
    };

    ResolvedAnimationConfig {
        mode,
        device: bag.device.clone().unwrap_or_else(|| default_device.to_string()),
        fps: bag.fps.unwrap_or(30),
        brightness: bag.brightness.unwrap_or(200),
        params,
    }
}

pub fn print_version() {
    println!("kbdanim {VERSION}");
}

pub fn print_usage() {
    println!(
        "{}",
        r##"kbdanim - professional single-zone keyboard backlight animation engine

USAGE:
    kbdanim <COMMAND> [OPTIONS]

COMMANDS:
    run              Run an animation in the foreground (Ctrl+C to stop)
    daemon           Run the animation engine as a foreground daemon
    stop             Stop a running daemon
    status           Show daemon status
    list             List all available animations
    search <term>    Search animations by name or description
    color <spec>     Set the LED color (name, #hex, or r,g,b)
    brightness <n>   Set the brightness (0-255)
    mode <name>      Change the running daemon's animation mode
    static           Set a static color (see --color/--brightness)
    off              Turn the keyboard LED off
    gaming           Apply the built-in "gaming" preset
    night            Apply the built-in "night" preset
    preset <cmd>     preset list | preset run <name> | preset create
    profile <cmd>    profile list | profile switch <name>
    power-profile    power-profile ac | power-profile battery
    random-mode      random-mode on | random-mode off
    playlist run     Run a playlist of animations in sequence
    interactive      Simple terminal-based live control
    doctor           Diagnose common setup problems
    config           config show | config path | config edit
    completions      completions bash | zsh | fish
    --version, -V    Print the version
    --help, -h        Print this help message

COMMON OPTIONS (for run/static/gaming/night/preset/playlist):
    --mode <name>          Animation to run (default: rainbow)
    --device <path>        LED sysfs directory (default: /sys/class/leds/rgb:kbd_backlight)
    --speed <0.1-10.0>      Animation speed multiplier (default: 1.0)
    --fps <1-240>           Frames per second (default: 30)
    --brightness <0-255>    Global brightness (default: 200)
    --color <spec>          Primary color: name, #rrggbb, or r,g,b
    --color2 <spec>         Secondary color (used by some animations)
    --easing <name>         linear|smoothstep|ease-in|ease-out|ease-in-out|sine|quadratic|cubic
    --period <secs>         Base cycle/pulse duration
    --randomness <0-1>      How chaotic stochastic animations feel
    --saturation <0-1>      Color saturation knob
    --temperature <0-1>     Warm/cool knob (a few animations)
    --intensity <n>         General intensity/flicker-speed multiplier
    --palette <name>        Built-in or user palette (see config/palettes)
    --seed <n>              Fix the RNG seed for reproducible output
    -v, --verbose           Increase log verbosity (repeatable)
    -q, --quiet             Only log errors

EXAMPLES:
    kbdanim list
    sudo kbdanim run --mode rainbow
    sudo kbdanim run --mode fire --speed 1.5
    sudo kbdanim static --color "#00aaff"
    kbdanim daemon
    kbdanim status
    kbdanim mode rainbow
    kbdanim preset run gaming
"##
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_basic_flags() {
        let bag = parse_flags(&args(&["--mode", "fire", "--speed", "1.5", "--fps", "60"])).unwrap();
        assert_eq!(bag.mode.as_deref(), Some("fire"));
        assert_eq!(bag.speed, Some(1.5));
        assert_eq!(bag.fps, Some(60));
    }

    #[test]
    fn parses_color_flags() {
        let bag = parse_flags(&args(&["--color", "red", "--color2", "#00ff00"])).unwrap();
        assert_eq!(bag.color, Some(Color::new(255, 0, 0)));
        assert_eq!(bag.color2, Some(Color::new(0, 255, 0)));
    }

    #[test]
    fn collects_positionals() {
        let bag = parse_flags(&args(&["run", "gaming"])).unwrap();
        assert_eq!(bag.positionals, vec!["run".to_string(), "gaming".to_string()]);
    }

    #[test]
    fn rejects_unknown_flag() {
        assert!(parse_flags(&args(&["--not-a-real-flag", "x"])).is_err());
    }

    #[test]
    fn rejects_missing_value() {
        assert!(parse_flags(&args(&["--speed"])).is_err());
    }

    #[test]
    fn rejects_out_of_range_speed() {
        assert!(parse_flags(&args(&["--speed", "99"])).is_err());
    }

    #[test]
    fn rejects_out_of_range_fps() {
        assert!(parse_flags(&args(&["--fps", "0"])).is_err());
    }

    #[test]
    fn rejects_out_of_range_brightness() {
        assert!(parse_flags(&args(&["--brightness", "500"])).is_err());
    }

    #[test]
    fn resolve_applies_mode_defaults_when_color_omitted() {
        let bag = parse_flags(&args(&["--mode", "police"])).unwrap();
        let resolved = resolve_animation_config(&bag, "/sys/class/leds/rgb:kbd_backlight");
        assert_eq!(resolved.params.color, Color::new(255, 0, 0));
    }

    #[test]
    fn resolve_respects_explicit_color() {
        let bag = parse_flags(&args(&["--mode", "police", "--color", "white"])).unwrap();
        let resolved = resolve_animation_config(&bag, "/dev/null");
        assert_eq!(resolved.params.color, Color::new(255, 255, 255));
    }

    #[test]
    fn resolve_defaults_mode_to_rainbow() {
        let bag = parse_flags(&args(&[])).unwrap();
        let resolved = resolve_animation_config(&bag, "/dev/null");
        assert_eq!(resolved.mode, "rainbow");
    }
}
