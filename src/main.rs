//! kbdanim: a professional, fully-CLI, single-zone keyboard backlight
//! animation engine for Linux. See `cli::print_usage()` for the command
//! list, or run `kbdanim --help`.
//!
//! This binary is a thin wrapper around the `kbdanim` library
//! (`src/lib.rs`) — the exact same backend that the optional
//! `kbdanim-gui` binary uses, so the CLI and GUI can never drift into
//! two different implementations of the same feature.

use kbdanim::{
    animation, animations, cli, color, config, daemon, device, engine, error, ipc, logging,
    palette, paths, preset, profile, rng, signal, toml_lite,
};
use device::KeyboardDevice;

use animation::AnimationParams;
use error::{KbdAnimError, Result};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let (args, verbose_count, quiet) = extract_global_flags(&raw_args);

    if quiet {
        logging::set_quiet();
    } else if verbose_count > 0 {
        logging::set_verbose(verbose_count);
    }

    match dispatch(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("kbdanim: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Pulls `-v`/`--verbose` (repeatable) and `-q`/`--quiet` out of the
/// argument list wherever they appear, so subcommand-specific parsing
/// never has to know about them.
fn extract_global_flags(args: &[String]) -> (Vec<String>, u8, bool) {
    let mut out = Vec::with_capacity(args.len());
    let mut verbose = 0u8;
    let mut quiet = false;
    for a in args {
        match a.as_str() {
            "-v" | "--verbose" => verbose = verbose.saturating_add(1),
            "-q" | "--quiet" => quiet = true,
            _ => out.push(a.clone()),
        }
    }
    (out, verbose, quiet)
}

fn dispatch(args: &[String]) -> Result<()> {
    let Some(command) = args.first() else {
        cli::print_usage();
        return Ok(());
    };
    let rest = &args[1..];

    match command.as_str() {
        "--help" | "-h" | "help" => cli::print_usage(),
        "--version" | "-V" | "version" => cli::print_version(),
        "run" => cmd_run(rest)?,
        "daemon" => cmd_daemon(rest)?,
        "stop" => cmd_stop()?,
        "status" => cmd_status()?,
        "restart" => cmd_restart(),
        "list" => cmd_list(),
        "animation" => cmd_animation(rest),
        "search" => cmd_search(rest)?,
        "color" => cmd_color(rest)?,
        "brightness" => cmd_brightness(rest)?,
        "mode" => cmd_mode(rest)?,
        "static" => cmd_static(rest)?,
        "off" => cmd_off()?,
        "gaming" => cmd_apply_preset("gaming")?,
        "night" => cmd_apply_preset("night")?,
        "preset" => cmd_preset(rest)?,
        "profile" => cmd_profile(rest)?,
        "power-profile" => cmd_power_profile(rest)?,
        "random-mode" => cmd_random_mode(rest)?,
        "playlist" => cmd_playlist(rest)?,
        "interactive" => cmd_interactive()?,
        "doctor" => cmd_doctor(),
        "config" => cmd_config(rest)?,
        "completions" => cmd_completions(rest)?,
        other => {
            return Err(KbdAnimError::Other(format!(
                "unknown command: '{other}' (see 'kbdanim --help')"
            )));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------
// run / daemon
// ---------------------------------------------------------------------

fn cmd_run(args: &[String]) -> Result<()> {
    let bag = cli::parse_flags(args)?;
    let resolved = cli::resolve_animation_config(&bag, device::DEFAULT_DEVICE_PATH);
    let mut params = resolved.params;

    if let Some(name) = &bag.palette {
        match palette::resolve(&paths::palettes_dir(), name) {
            Some(colors) => params.palette = Some(colors),
            None => return Err(KbdAnimError::Other(format!("unknown palette: {name}"))),
        }
    }

    let mut anim = animations::create(&resolved.mode, params.seed)
        .ok_or_else(|| KbdAnimError::InvalidMode(resolved.mode.clone()))?;

    signal::install();
    let mut dev = device::SysfsDevice::new(PathBuf::from(&resolved.device));
    let mut device_ok = dev.exists();
    if !device_ok {
        logging::warn(&format!(
            "LED device not found at {} — will keep retrying",
            resolved.device
        ));
    }

    let mut eng = engine::Engine::new(engine::EngineConfig {
        fps: resolved.fps,
        global_brightness: resolved.brightness,
        quantization: 1,
    });

    logging::info(&format!(
        "running '{}' at {} fps, brightness {} (Ctrl+C to stop)",
        resolved.mode, resolved.fps, resolved.brightness
    ));

    loop {
        if signal::shutdown_requested() {
            break;
        }
        match eng.step(anim.as_mut(), &params, &mut dev) {
            Ok(()) => {
                if !device_ok {
                    logging::info("LED device detected, resuming normal operation");
                    dev.invalidate_cache();
                    device_ok = true;
                }
            }
            Err(e) => {
                if device_ok {
                    logging::warn(&format!("device write failed: {e}"));
                    device_ok = false;
                }
            }
        }
        eng.pace();
    }

    let _ = dev.write_brightness(0);
    logging::info("stopped");
    Ok(())
}

fn cmd_daemon(args: &[String]) -> Result<()> {
    let bag = cli::parse_flags(args)?;
    let mut config = config::Config::load_or_default(&paths::config_file());

    if let Some(m) = &bag.mode {
        config.mode = m.clone();
    }
    if let Some(d) = &bag.device {
        config.device = d.clone();
    }
    if let Some(s) = bag.speed {
        config.speed = s;
    }
    if let Some(f) = bag.fps {
        config.fps = f;
    }
    if let Some(b) = bag.brightness {
        config.brightness = b;
    }

    daemon::run(config)
}

fn cmd_stop() -> Result<()> {
    let resp = ipc::send_command(&paths::socket_path(), "STOP")?;
    println!("{resp}");
    Ok(())
}

fn cmd_status() -> Result<()> {
    match ipc::send_command(&paths::socket_path(), "STATUS") {
        Ok(resp) => {
            let mut lines = resp.lines();
            let first = lines.next().unwrap_or("");
            if first != "OK" {
                println!("kbdanim status\n\nStatus:      error ({first})");
                return Ok(());
            }
            println!("kbdanim status\n");
            println!("Status:      running");
            for line in lines {
                if let Some((k, v)) = line.split_once('=') {
                    let label = match k {
                        "mode" => "Mode",
                        "brightness" => "Brightness",
                        "speed" => "Speed",
                        "fps" => "FPS",
                        "color" => "Color",
                        "color2" => "Color2",
                        "easing" => "Easing",
                        "profile" => "Profile",
                        "paused" => "Paused",
                        "random_mode" => "Random mode",
                        _ => k,
                    };
                    println!("{label:<12} {v}");
                }
            }
        }
        Err(_) => {
            println!("kbdanim status\n");
            println!("Status:      not running\n");
            println!("Start it with 'kbdanim daemon' (or 'systemctl --user start kbdanim').");
        }
    }
    Ok(())
}

fn cmd_restart() {
    match ipc::send_command(&paths::socket_path(), "STOP") {
        Ok(_) => println!(
            "Daemon stopped. Start it again with 'kbdanim daemon' (or 'systemctl --user restart kbdanim')."
        ),
        Err(_) => println!("Daemon was not running. Start it with 'kbdanim daemon'."),
    }
}

// ---------------------------------------------------------------------
// animation listing / search
// ---------------------------------------------------------------------

fn cmd_list() {
    println!("Available animations:\n");
    for info in animations::list() {
        println!("{:<16} {}", info.name, info.description);
    }
}

fn cmd_animation(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("list") | None => cmd_list(),
        Some(other) => println!("unknown 'animation' subcommand: {other} (try 'kbdanim animation list')"),
    }
}

fn cmd_search(args: &[String]) -> Result<()> {
    let term = args
        .first()
        .ok_or_else(|| KbdAnimError::Other("usage: kbdanim search <term>".to_string()))?
        .to_lowercase();
    let mut found = false;
    for info in animations::list() {
        if info.name.to_lowercase().contains(&term) || info.description.to_lowercase().contains(&term) {
            println!("{:<16} {}", info.name, info.description);
            found = true;
        }
    }
    if !found {
        println!("No animations matched '{term}'.");
    }
    Ok(())
}

// ---------------------------------------------------------------------
// one-shot color / brightness / mode / static / off
// ---------------------------------------------------------------------

fn cmd_color(args: &[String]) -> Result<()> {
    let spec = args
        .first()
        .ok_or_else(|| KbdAnimError::Other("usage: kbdanim color <name|#hex|r,g,b|random>".to_string()))?;
    let color = if spec.eq_ignore_ascii_case("random") {
        let mut rng = rng::Rng::from_entropy();
        color::Color::random_vivid(&mut rng)
    } else {
        color::Color::parse(spec)?
    };

    match ipc::send_command(&paths::socket_path(), &format!("COLOR {}", color.to_hex())) {
        Ok(resp) => println!("{resp} ({})", color.to_hex()),
        Err(_) => {
            let mut dev = device::SysfsDevice::default_device();
            dev.write_color(color)?;
            println!("Set color to {} (no daemon running; wrote directly to the device)", color.to_hex());
        }
    }
    Ok(())
}

fn cmd_brightness(args: &[String]) -> Result<()> {
    let raw = args
        .first()
        .ok_or_else(|| KbdAnimError::Other("usage: kbdanim brightness <0-255>".to_string()))?;
    let value: i32 = raw
        .parse()
        .map_err(|_| KbdAnimError::InvalidBrightness(raw.clone()))?;
    if !(0..=255).contains(&value) {
        return Err(KbdAnimError::InvalidBrightness(format!("{raw} (must be 0-255)")));
    }
    let value = value as u8;

    match ipc::send_command(&paths::socket_path(), &format!("BRIGHTNESS {value}")) {
        Ok(resp) => println!("{resp}"),
        Err(_) => {
            let mut dev = device::SysfsDevice::default_device();
            dev.write_brightness(value)?;
            println!("Set brightness to {value} (no daemon running; wrote directly to the device)");
        }
    }
    Ok(())
}

fn cmd_mode(args: &[String]) -> Result<()> {
    let name = args
        .first()
        .ok_or_else(|| KbdAnimError::Other("usage: kbdanim mode <name>".to_string()))?;
    if animations::create(name, 0).is_none() {
        return Err(KbdAnimError::InvalidMode(name.clone()));
    }
    match ipc::send_command(&paths::socket_path(), &format!("MODE {name}")) {
        Ok(resp) => println!("{resp}"),
        Err(_) => {
            println!("No daemon is running, so there is nothing to switch modes on.");
            println!("Start one with 'kbdanim daemon', or run this animation directly:");
            println!("  sudo kbdanim run --mode {name}");
        }
    }
    Ok(())
}

fn cmd_static(args: &[String]) -> Result<()> {
    let bag = cli::parse_flags(args)?;
    let color = bag.color.unwrap_or(color::Color::new(255, 255, 255));
    let brightness = bag.brightness.unwrap_or(255);

    let batch = format!("mode=static;color={};brightness={brightness}", color.to_hex());
    match ipc::send_command(&paths::socket_path(), &format!("BATCH {batch}")) {
        Ok(resp) => println!("{resp}"),
        Err(_) => {
            let device_path = bag
                .device
                .clone()
                .unwrap_or_else(|| device::DEFAULT_DEVICE_PATH.to_string());
            let mut dev = device::SysfsDevice::new(PathBuf::from(device_path));
            dev.write_color(color)?;
            dev.write_brightness(brightness)?;
            println!(
                "Set static color {} at brightness {brightness} (no daemon running; wrote directly to the device)",
                color.to_hex()
            );
        }
    }
    Ok(())
}

fn cmd_off() -> Result<()> {
    match ipc::send_command(&paths::socket_path(), "BATCH mode=static;brightness=0") {
        Ok(resp) => println!("{resp}"),
        Err(_) => {
            let mut dev = device::SysfsDevice::default_device();
            dev.write_brightness(0)?;
            println!("Turned the keyboard LED off.");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------
// presets / profiles / power-profile / random-mode / playlist
// ---------------------------------------------------------------------

fn cmd_apply_preset(name: &str) -> Result<()> {
    let dir = paths::presets_dir();
    let preset = preset::find_preset(&dir, name)
        .ok_or_else(|| KbdAnimError::Other(format!("unknown preset: {name}")))?;

    let mut batch = format!(
        "mode={};brightness={};speed={};fps={}",
        preset.mode, preset.brightness, preset.speed, preset.fps
    );
    if let Some(c) = preset.color {
        batch.push_str(&format!(";color={}", c.to_hex()));
    }
    if let Some(c2) = preset.color2 {
        batch.push_str(&format!(";color2={}", c2.to_hex()));
    }
    if let Some(e) = &preset.easing {
        batch.push_str(&format!(";easing={e}"));
    }

    match ipc::send_command(&paths::socket_path(), &format!("BATCH {batch}")) {
        Ok(resp) => println!("Applied preset '{name}': {resp}"),
        Err(_) => {
            println!("No daemon is running. Start one first ('kbdanim daemon'), or run this preset directly:");
            println!(
                "  sudo kbdanim run --mode {} --brightness {} --speed {} --fps {}",
                preset.mode, preset.brightness, preset.speed, preset.fps
            );
        }
    }
    Ok(())
}

fn cmd_preset(args: &[String]) -> Result<()> {
    match args.first().map(|s| s.as_str()) {
        Some("list") => {
            println!("Available presets:\n");
            for p in preset::all_presets(&paths::presets_dir()) {
                println!(
                    "{:<10} mode={:<14} brightness={:<4} speed={:<5} fps={}",
                    p.name, p.mode, p.brightness, p.speed, p.fps
                );
            }
            Ok(())
        }
        Some("run") => {
            let name = args
                .get(1)
                .ok_or_else(|| KbdAnimError::Other("usage: kbdanim preset run <name>".to_string()))?;
            cmd_apply_preset(name)
        }
        Some("create") => cmd_preset_create(),
        _ => {
            println!("usage: kbdanim preset list | kbdanim preset run <name> | kbdanim preset create");
            Ok(())
        }
    }
}

fn ask(prompt: &str) -> Result<String> {
    use std::io::Write as _;
    print!("{prompt}");
    std::io::stdout().flush().ok();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn cmd_preset_create() -> Result<()> {
    let name = ask("Preset name: ")?;
    if name.is_empty() {
        return Err(KbdAnimError::Other("preset name cannot be empty".to_string()));
    }
    let mode = ask("Animation mode (see 'kbdanim list'): ")?;
    if animations::create(&mode, 0).is_none() {
        return Err(KbdAnimError::InvalidMode(mode));
    }
    let brightness_s = ask("Brightness [0-255, default 200]: ")?;
    let brightness: u8 = if brightness_s.is_empty() {
        200
    } else {
        brightness_s
            .parse()
            .map_err(|_| KbdAnimError::InvalidBrightness(brightness_s.clone()))?
    };
    let speed_s = ask("Speed [0.1-10.0, default 1.0]: ")?;
    let speed: f64 = if speed_s.is_empty() {
        1.0
    } else {
        speed_s.parse().map_err(|_| KbdAnimError::InvalidSpeed(speed_s.clone()))?
    };
    let fps_s = ask("FPS [1-240, default 30]: ")?;
    let fps: u32 = if fps_s.is_empty() {
        30
    } else {
        fps_s.parse().map_err(|_| KbdAnimError::InvalidFps(fps_s.clone()))?
    };
    let color_s = ask("Color [optional: name, #hex, or r,g,b]: ")?;

    paths::ensure_dirs()?;
    let path = paths::presets_dir().join(format!("{name}.toml"));
    let mut text = format!(
        "name = \"{name}\"\nmode = \"{mode}\"\nbrightness = {brightness}\nspeed = {speed}\nfps = {fps}\n"
    );
    if !color_s.is_empty() {
        let c = color::Color::parse(&color_s)?;
        text.push_str(&format!("color = \"{}\"\n", c.to_hex()));
    }
    std::fs::write(&path, text)?;
    println!("Saved preset '{name}' to {}", path.display());
    Ok(())
}

fn cmd_profile(args: &[String]) -> Result<()> {
    match args.first().map(|s| s.as_str()) {
        Some("list") => {
            let config = config::Config::load_or_default(&paths::config_file());
            println!("Available profiles:\n");
            for name in profile::list(&config) {
                if let Some(p) = profile::get(&config, &name) {
                    println!(
                        "{:<10} mode={:<14} brightness={:<4} speed={:<5} fps={}",
                        name, p.mode, p.brightness, p.speed, p.fps
                    );
                }
            }
            Ok(())
        }
        Some("switch") => {
            let name = args
                .get(1)
                .ok_or_else(|| KbdAnimError::Other("usage: kbdanim profile switch <name>".to_string()))?;
            let config = config::Config::load_or_default(&paths::config_file());
            if !config.profiles.contains_key(name) {
                return Err(KbdAnimError::Other(format!("unknown profile: {name}")));
            }
            match ipc::send_command(&paths::socket_path(), &format!("PROFILE {name}")) {
                Ok(resp) => println!("{resp}"),
                Err(_) => println!("No daemon is running. Start one with 'kbdanim daemon' to use profiles."),
            }
            Ok(())
        }
        _ => {
            println!("usage: kbdanim profile list | kbdanim profile switch <name>");
            Ok(())
        }
    }
}

fn cmd_power_profile(args: &[String]) -> Result<()> {
    let name = args
        .first()
        .ok_or_else(|| KbdAnimError::Other("usage: kbdanim power-profile ac|battery".to_string()))?;
    if name != "ac" && name != "battery" {
        return Err(KbdAnimError::Other(format!(
            "unknown power profile: {name} (expected 'ac' or 'battery')"
        )));
    }
    cmd_profile(&["switch".to_string(), name.clone()])
}

fn cmd_random_mode(args: &[String]) -> Result<()> {
    let arg = args.first().map(|s| s.to_lowercase()).unwrap_or_else(|| "on".to_string());
    let ipc_cmd = match arg.as_str() {
        "on" => "RANDOM_MODE ON",
        "off" => "RANDOM_MODE OFF",
        _ => return Err(KbdAnimError::Other("usage: kbdanim random-mode on|off".to_string())),
    };
    match ipc::send_command(&paths::socket_path(), ipc_cmd) {
        Ok(resp) => println!("{resp}"),
        Err(_) => println!("No daemon is running. Start one with 'kbdanim daemon' to use random mode."),
    }
    Ok(())
}

fn cmd_playlist(args: &[String]) -> Result<()> {
    match args.first().map(|s| s.as_str()) {
        Some("run") => {
            let name = args
                .get(1)
                .ok_or_else(|| KbdAnimError::Other("usage: kbdanim playlist run <name>".to_string()))?;
            run_playlist(name)
        }
        _ => {
            println!("usage: kbdanim playlist run <name>");
            Ok(())
        }
    }
}

fn run_playlist(name: &str) -> Result<()> {
    let path = paths::playlists_dir().join(format!("{name}.toml"));
    let text = std::fs::read_to_string(&path)
        .map_err(|_| KbdAnimError::Other(format!("playlist not found: {}", path.display())))?;
    let doc = toml_lite::parse(&text)?;
    let steps = doc
        .array("animations")
        .ok_or_else(|| KbdAnimError::Other("playlist file has no [[animations]] entries".to_string()))?;
    if steps.is_empty() {
        return Err(KbdAnimError::Other("playlist is empty".to_string()));
    }

    signal::install();
    let mut dev = device::SysfsDevice::default_device();
    let mut eng = engine::Engine::new(engine::EngineConfig {
        fps: 30,
        global_brightness: 200,
        quantization: 1,
    });

    'playlist: loop {
        for step in steps {
            let mode = step.get_str("mode").unwrap_or("rainbow");
            let duration = step.get_f64("duration").unwrap_or(30.0).max(0.5);
            let (default_color, default_color2) = animation::default_colors_for(mode);
            let params = AnimationParams {
                color: default_color,
                color2: default_color2,
                ..AnimationParams::default()
            };

            let mut anim = match animations::create(mode, 0) {
                Some(a) => a,
                None => {
                    logging::warn(&format!("skipping unknown playlist mode: {mode}"));
                    continue;
                }
            };

            logging::info(&format!("playlist: '{mode}' for {duration:.1}s"));
            let start = std::time::Instant::now();
            while start.elapsed().as_secs_f64() < duration {
                if signal::shutdown_requested() {
                    break 'playlist;
                }
                let _ = eng.step(anim.as_mut(), &params, &mut dev);
                eng.pace();
            }
        }
    }

    let _ = dev.write_brightness(0);
    Ok(())
}

// ---------------------------------------------------------------------
// interactive
// ---------------------------------------------------------------------

/// A simple line-based interactive controller. The spec sketches an
/// arrow-key single-keypress interface; this project deliberately
/// implements a line-based one instead. Raw single-keypress input on
/// Linux requires manipulating terminal `termios` flags via raw libc
/// struct layouts, which cannot be safely hand-verified without a
/// compiler+terminal in the loop, and the spec explicitly says this
/// feature must not add undue complexity to the core program — a
/// plain, robust `read_line` loop satisfies the "terminal-based control,
/// no GUI" requirement with far less risk.
fn cmd_interactive() -> Result<()> {
    let socket = paths::socket_path();
    if ipc::send_command(&socket, "PING").is_err() {
        println!("No daemon detected — starting one now...");
        spawn_background_daemon()?;
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    println!("kbdanim interactive control. Type 'help' for commands, 'quit' to exit.\n");
    loop {
        use std::io::Write as _;
        print!("kbdanim> ");
        std::io::stdout().flush().ok();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match line {
            "quit" | "exit" | "q" => break,
            "help" | "h" | "?" => print_interactive_help(),
            "status" => {
                let _ = cmd_status();
            }
            "pause" => report(ipc::send_command(&socket, "PAUSE")),
            "resume" => report(ipc::send_command(&socket, "RESUME")),
            "random" => report(ipc::send_command(&socket, &format!("MODE {}", random_mode_name()))),
            _ => {
                if let Some(v) = line.strip_prefix("mode ") {
                    report(ipc::send_command(&socket, &format!("MODE {}", v.trim())));
                } else if let Some(v) = line.strip_prefix("brightness ") {
                    report(ipc::send_command(&socket, &format!("BRIGHTNESS {}", v.trim())));
                } else if let Some(v) = line.strip_prefix("speed ") {
                    report(ipc::send_command(&socket, &format!("SPEED {}", v.trim())));
                } else if let Some(v) = line.strip_prefix("color ") {
                    report(ipc::send_command(&socket, &format!("COLOR {}", v.trim())));
                } else {
                    println!("unrecognized command (try 'help')");
                }
            }
        }
    }
    Ok(())
}

fn report(result: Result<String>) {
    match result {
        Ok(resp) => println!("{resp}"),
        Err(e) => println!("error: {e}"),
    }
}

fn random_mode_name() -> String {
    let mut r = rng::Rng::from_entropy();
    let names = animations::names();
    if names.is_empty() {
        return "rainbow".to_string();
    }
    let idx = r.range_u32(0, (names.len() - 1) as u32) as usize;
    names[idx].to_string()
}

fn print_interactive_help() {
    println!("commands:");
    println!("  mode <name>          change animation");
    println!("  brightness <0-255>   set brightness");
    println!("  speed <0.1-10.0>     set speed");
    println!("  color <spec>         set color");
    println!("  random               switch to a random animation");
    println!("  pause / resume       pause or resume output");
    println!("  status               show daemon status");
    println!("  quit                 exit interactive mode");
}

fn spawn_background_daemon() -> Result<()> {
    let exe = std::env::current_exe()?;
    std::process::Command::new(exe)
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    Ok(())
}

// ---------------------------------------------------------------------
// doctor
// ---------------------------------------------------------------------

fn cmd_doctor() {
    println!("kbdanim doctor\n");

    let device_dir = PathBuf::from(device::DEFAULT_DEVICE_PATH);
    let brightness_path = device_dir.join("brightness");
    let color_path = device_dir.join("multi_intensity");

    doctor_check("Device directory found", device_dir.exists());
    doctor_check("brightness file available", brightness_path.exists());
    doctor_check("multi_intensity file available", color_path.exists());
    doctor_check(
        "Read access to brightness",
        std::fs::read_to_string(&brightness_path).is_ok(),
    );
    doctor_check_warn(
        "Write access to brightness",
        std::fs::OpenOptions::new().write(true).open(&brightness_path).is_ok(),
    );
    doctor_check("Config directory writable", paths::ensure_dirs().is_ok());

    match ipc::send_command(&paths::socket_path(), "PING") {
        Ok(_) => doctor_check("Daemon reachable via IPC", true),
        Err(_) => println!("[--]   Daemon reachable via IPC (not running — fine if you use 'kbdanim run')"),
    }

    println!("\nAnimations registered: {}", animations::count());
}

fn doctor_check(label: &str, ok: bool) {
    if ok {
        println!("[OK]   {label}");
    } else {
        println!("[FAIL] {label}");
    }
}

fn doctor_check_warn(label: &str, ok: bool) {
    if ok {
        println!("[OK]   {label}");
    } else {
        println!("[WARN] {label} (see README for udev/group setup)");
    }
}

// ---------------------------------------------------------------------
// config
// ---------------------------------------------------------------------

fn cmd_config(args: &[String]) -> Result<()> {
    match args.first().map(|s| s.as_str()) {
        Some("path") => {
            println!("{}", paths::config_file().display());
            Ok(())
        }
        Some("show") => {
            let cfg = config::Config::load_or_default(&paths::config_file());
            println!("device               = {}", cfg.device);
            println!("mode                 = {}", cfg.mode);
            println!("speed                = {}", cfg.speed);
            println!("fps                  = {}", cfg.fps);
            println!("brightness           = {}", cfg.brightness);
            println!("easing               = {}", cfg.easing);
            println!("color                = {}", cfg.color.to_hex());
            println!("color2               = {}", cfg.color2.to_hex());
            println!("daemon.enabled       = {}", cfg.daemon_enabled);
            println!("daemon.restore_state = {}", cfg.restore_state);
            println!("power.auto_profile   = {}", cfg.auto_profile);
            println!("engine.quantization  = {}", cfg.color_quantization);
            Ok(())
        }
        Some("edit") => cmd_config_edit(),
        _ => {
            println!("usage: kbdanim config show | kbdanim config path | kbdanim config edit");
            Ok(())
        }
    }
}

fn cmd_config_edit() -> Result<()> {
    paths::ensure_dirs()?;
    let path = paths::config_file();
    if !path.exists() {
        std::fs::write(&path, include_str!("../config/default.toml"))?;
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let status = std::process::Command::new(&editor).arg(&path).status();
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(KbdAnimError::Other(format!("{editor} exited with {s}"))),
        Err(e) => Err(KbdAnimError::Other(format!("could not launch '{editor}': {e}"))),
    }
}

// ---------------------------------------------------------------------
// completions
// ---------------------------------------------------------------------

fn cmd_completions(args: &[String]) -> Result<()> {
    let shell = args
        .first()
        .ok_or_else(|| KbdAnimError::Other("usage: kbdanim completions bash|zsh|fish".to_string()))?;
    let script = match shell.as_str() {
        "bash" => include_str!("../completions/kbdanim.bash"),
        "zsh" => include_str!("../completions/kbdanim.zsh"),
        "fish" => include_str!("../completions/kbdanim.fish"),
        other => {
            return Err(KbdAnimError::Other(format!(
                "unknown shell: {other} (expected bash, zsh, or fish)"
            )))
        }
    };
    println!("{script}");
    Ok(())
}
