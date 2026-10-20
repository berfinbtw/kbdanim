//! The "service layer" between the GUI and the existing kbdanim
//! backend (spec: `GUI -> Application/Service layer -> existing backend
//! -> Device`).
//!
//! Nothing in `src/gui/` ever opens a sysfs file or runs an animation
//! loop directly. Every read goes through the same `kbdanim` library
//! the CLI uses (`config`, `preset`, `palette`, `paths`, `animations`,
//! `power`, `device`), and every write to the LED goes through the
//! running daemon over the exact same Unix-socket IPC protocol the CLI
//! uses (`kbdanim::ipc`). This is what "reuse the backend, don't
//! rewrite it" means in practice.

use kbdanim::color::Color;
use kbdanim::config::Config;
use kbdanim::device::SysfsDevice;
use kbdanim::error::KbdAnimError;
use kbdanim::{animations, ipc, palette, paths, power, preset, toml_lite};
use std::path::PathBuf;
use std::time::Duration;

/// A snapshot of the running daemon's state, as reported over IPC.
/// `running = false` means no daemon answered (everything else is then
/// just a harmless placeholder — the GUI must not show stale numbers
/// as if they were live).
#[derive(Debug, Clone)]
pub struct DaemonStatus {
    pub running: bool,
    pub mode: String,
    pub brightness: u8,
    pub speed: f64,
    pub fps: u32,
    pub color: Color,
    pub color2: Color,
    pub easing: String,
    pub profile: String,
    pub paused: bool,
    pub random_mode: bool,
}

impl Default for DaemonStatus {
    fn default() -> Self {
        DaemonStatus {
            running: false,
            mode: "rainbow".to_string(),
            brightness: 200,
            speed: 1.0,
            fps: 30,
            color: Color::new(0, 170, 255),
            color2: Color::new(255, 0, 128),
            easing: "smoothstep".to_string(),
            profile: "default".to_string(),
            paused: false,
            random_mode: false,
        }
    }
}

fn parse_status(resp: &str) -> DaemonStatus {
    let mut st = DaemonStatus {
        running: true,
        ..DaemonStatus::default()
    };
    for line in resp.lines().skip(1) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "mode" => st.mode = value.to_string(),
            "brightness" => st.brightness = value.parse().unwrap_or(st.brightness),
            "speed" => st.speed = value.parse().unwrap_or(st.speed),
            "fps" => st.fps = value.parse().unwrap_or(st.fps),
            "color" => {
                if let Ok(c) = Color::parse(value) {
                    st.color = c;
                }
            }
            "color2" => {
                if let Ok(c) = Color::parse(value) {
                    st.color2 = c;
                }
            }
            "easing" => st.easing = value.to_string(),
            "profile" => st.profile = value.to_string(),
            "paused" => st.paused = value == "true",
            "random_mode" => st.random_mode = value == "true",
            _ => {}
        }
    }
    st
}

/// The GUI's connection to the rest of kbdanim. Owns the loaded config
/// and the last error to display, and wraps every daemon interaction.
pub struct GuiBackend {
    socket_path: PathBuf,
    pub config: Config,
    pub last_error: Option<String>,
}

impl GuiBackend {
    pub fn new() -> Self {
        GuiBackend {
            socket_path: paths::socket_path(),
            config: Config::load_or_default(&paths::config_file()),
            last_error: None,
        }
    }

    pub fn config_path(&self) -> PathBuf {
        paths::config_file()
    }

    pub fn reload_config(&mut self) {
        self.config = Config::load_or_default(&paths::config_file());
    }

    pub fn save_config(&mut self) {
        if let Err(e) = self.config.save(&paths::config_file()) {
            self.report(e);
        }
    }

    /// Whether the configured LED device is present on this system.
    pub fn device_present(&self) -> bool {
        SysfsDevice::new(PathBuf::from(&self.config.device)).exists()
    }

    /// Whether the process can currently write to the device's
    /// brightness file (a good proxy for "udev rule installed
    /// correctly" without needing to actually flip the light).
    pub fn device_writable(&self) -> bool {
        let path = PathBuf::from(&self.config.device).join("brightness");
        std::fs::OpenOptions::new().write(true).open(path).is_ok()
    }

    pub fn is_daemon_running(&self) -> bool {
        ipc::send_command(&self.socket_path, "PING").is_ok()
    }

    /// Polls the daemon for its current state. Cheap: one short-lived
    /// Unix-socket round trip. Called on a timer from the GUI, never in
    /// a tight loop (see `app.rs`'s poll interval).
    pub fn fetch_status(&mut self) -> DaemonStatus {
        match ipc::send_command(&self.socket_path, "STATUS") {
            Ok(resp) if resp.starts_with("OK") => parse_status(&resp),
            Ok(_) => DaemonStatus::default(),
            Err(_) => DaemonStatus::default(),
        }
    }

    /// Starts `kbdanim daemon` in the background if it isn't already
    /// running, reusing the real daemon binary (never a second
    /// animation engine implemented in the GUI). Blocks briefly the
    /// first time only, waiting for the socket to come up.
    pub fn ensure_daemon_running(&mut self) -> bool {
        if self.is_daemon_running() {
            return true;
        }
        match spawn_daemon() {
            Ok(()) => {
                for _ in 0..20 {
                    std::thread::sleep(Duration::from_millis(50));
                    if self.is_daemon_running() {
                        return true;
                    }
                }
                self.last_error = Some(
                    "Started the daemon, but it did not respond in time.".to_string(),
                );
                false
            }
            Err(e) => {
                self.last_error = Some(format!("Could not start the kbdanim daemon: {e}"));
                false
            }
        }
    }

    pub fn stop_daemon(&mut self) {
        self.send("STOP");
    }

    fn send(&mut self, cmd: &str) {
        self.ensure_daemon_running();
        if let Err(e) = ipc::send_command(&self.socket_path, cmd) {
            self.report(e);
        }
    }

    pub fn set_mode(&mut self, mode: &str) {
        self.send(&format!("MODE {mode}"));
    }

    pub fn set_brightness(&mut self, v: u8) {
        self.send(&format!("BRIGHTNESS {v}"));
    }

    pub fn set_speed(&mut self, v: f64) {
        self.send(&format!("SPEED {v}"));
    }

    pub fn set_fps(&mut self, v: u32) {
        self.send(&format!("FPS {v}"));
    }

    pub fn set_color(&mut self, c: Color) {
        self.send(&format!("COLOR {}", c.to_hex()));
    }

    pub fn set_color2(&mut self, c: Color) {
        self.send(&format!("COLOR2 {}", c.to_hex()));
    }

    pub fn set_easing(&mut self, name: &str) {
        self.send(&format!("EASING {name}"));
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.send(if paused { "PAUSE" } else { "RESUME" });
    }

    pub fn set_random_mode(&mut self, on: bool) {
        self.send(if on { "RANDOM_MODE ON" } else { "RANDOM_MODE OFF" });
    }

    pub fn switch_profile(&mut self, name: &str) {
        self.send(&format!("PROFILE {name}"));
    }

    pub fn apply_batch(&mut self, batch: &str) {
        self.send(&format!("BATCH {batch}"));
    }

    /// Turns the light off without stopping the daemon (so the next
    /// animation change still applies instantly).
    pub fn turn_off(&mut self) {
        self.apply_batch("mode=static;brightness=0");
    }

    pub fn apply_preset(&mut self, name: &str) {
        if let Some(p) = preset::find_preset(&paths::presets_dir(), name) {
            let mut batch = format!(
                "mode={};brightness={};speed={};fps={}",
                p.mode, p.brightness, p.speed, p.fps
            );
            if let Some(c) = p.color {
                batch.push_str(&format!(";color={}", c.to_hex()));
            }
            if let Some(c2) = p.color2 {
                batch.push_str(&format!(";color2={}", c2.to_hex()));
            }
            if let Some(e) = &p.easing {
                batch.push_str(&format!(";easing={e}"));
            }
            self.apply_batch(&batch);
        }
    }

    pub fn animation_list(&self) -> Vec<(String, String)> {
        animations::list()
            .into_iter()
            .map(|i| (i.name.to_string(), i.description.to_string()))
            .collect()
    }

    pub fn presets(&self) -> Vec<preset::Preset> {
        preset::all_presets(&paths::presets_dir())
    }

    pub fn is_builtin_preset(&self, name: &str) -> bool {
        preset::builtin_presets().iter().any(|p| p.name == name)
            && !paths::presets_dir().join(format!("{name}.toml")).exists()
    }

    pub fn save_preset(
        &mut self,
        name: &str,
        mode: &str,
        brightness: u8,
        speed: f64,
        fps: u32,
        color: Option<Color>,
        color2: Option<Color>,
        easing: Option<&str>,
    ) {
        if let Err(e) = paths::ensure_dirs() {
            self.report(e);
            return;
        }
        let path = paths::presets_dir().join(format!("{name}.toml"));
        let mut text = format!(
            "name = \"{name}\"\nmode = \"{mode}\"\nbrightness = {brightness}\nspeed = {speed}\nfps = {fps}\n"
        );
        if let Some(c) = color {
            text.push_str(&format!("color = \"{}\"\n", c.to_hex()));
        }
        if let Some(c2) = color2 {
            text.push_str(&format!("color2 = \"{}\"\n", c2.to_hex()));
        }
        if let Some(e) = easing {
            text.push_str(&format!("easing = \"{e}\"\n"));
        }
        if let Err(e) = std::fs::write(&path, text) {
            self.report(KbdAnimError::from(e));
        }
    }

    pub fn delete_preset(&mut self, name: &str) {
        let path = paths::presets_dir().join(format!("{name}.toml"));
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                self.report(KbdAnimError::from(e));
            }
        }
    }

    pub fn palettes(&self) -> Vec<(String, Vec<Color>, bool)> {
        // (name, colors, is_user_defined)
        let mut out = Vec::new();
        for name in palette::builtin_names() {
            if let Some(colors) = palette::builtin_palette(name) {
                out.push((name.to_string(), colors, false));
            }
        }
        if let Ok(entries) = std::fs::read_dir(paths::palettes_dir()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if let Some(colors) = palette::load_user_palette(&paths::palettes_dir(), stem) {
                    out.retain(|(n, _, _)| n != stem);
                    out.push((stem.to_string(), colors, true));
                }
            }
        }
        out
    }

    pub fn save_palette(&mut self, name: &str, colors: &[Color]) {
        if let Err(e) = paths::ensure_dirs() {
            self.report(e);
            return;
        }
        let color_list: Vec<String> = colors.iter().map(|c| format!("\"{}\"", c.to_hex())).collect();
        let text = format!("name = \"{name}\"\ncolors = [{}]\n", color_list.join(", "));
        let path = paths::palettes_dir().join(format!("{name}.toml"));
        if let Err(e) = std::fs::write(&path, text) {
            self.report(KbdAnimError::from(e));
        }
    }

    pub fn delete_palette(&mut self, name: &str) {
        let path = paths::palettes_dir().join(format!("{name}.toml"));
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                self.report(KbdAnimError::from(e));
            }
        }
    }

    pub fn playlist_names(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(entries) = std::fs::read_dir(paths::playlists_dir()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        out.push(stem.to_string());
                    }
                }
            }
        }
        out.sort();
        out
    }

    pub fn load_playlist(&self, name: &str) -> Vec<(String, f64)> {
        let path = paths::playlists_dir().join(format!("{name}.toml"));
        let Ok(text) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        let Ok(doc) = toml_lite::parse(&text) else {
            return Vec::new();
        };
        let mut steps = Vec::new();
        if let Some(array) = doc.array("animations") {
            for section in array {
                let mode = section.get_str("mode").unwrap_or("rainbow").to_string();
                let duration = section.get_f64("duration").unwrap_or(30.0);
                steps.push((mode, duration));
            }
        }
        steps
    }

    pub fn save_playlist(&mut self, name: &str, steps: &[(String, f64)]) {
        if let Err(e) = paths::ensure_dirs() {
            self.report(e);
            return;
        }
        let mut text = String::new();
        for (mode, duration) in steps {
            text.push_str("[[animations]]\n");
            text.push_str(&format!("mode = \"{mode}\"\n"));
            text.push_str(&format!("duration = {duration}\n\n"));
        }
        let path = paths::playlists_dir().join(format!("{name}.toml"));
        if let Err(e) = std::fs::write(&path, text) {
            self.report(KbdAnimError::from(e));
        }
    }

    pub fn delete_playlist(&mut self, name: &str) {
        let path = paths::playlists_dir().join(format!("{name}.toml"));
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                self.report(KbdAnimError::from(e));
            }
        }
    }

    pub fn power_state(&self) -> power::PowerState {
        power::detect()
    }

    /// systemd `--user` unit control. These commands never need root:
    /// a user's own systemd instance manages units in their own
    /// session, so there is no password prompt to design around.
    pub fn systemd_is_active(&self) -> bool {
        run_systemctl(&["--user", "is-active", "kbdanim"])
            .map(|out| out.trim() == "active")
            .unwrap_or(false)
    }

    pub fn systemd_is_enabled(&self) -> bool {
        run_systemctl(&["--user", "is-enabled", "kbdanim"])
            .map(|out| out.trim() == "enabled")
            .unwrap_or(false)
    }

    pub fn systemd_start(&mut self) {
        self.run_systemctl_reporting(&["--user", "start", "kbdanim"]);
    }

    pub fn systemd_stop(&mut self) {
        self.run_systemctl_reporting(&["--user", "stop", "kbdanim"]);
    }

    pub fn systemd_enable(&mut self) {
        self.run_systemctl_reporting(&["--user", "enable", "kbdanim"]);
    }

    pub fn systemd_disable(&mut self) {
        self.run_systemctl_reporting(&["--user", "disable", "kbdanim"]);
    }

    fn run_systemctl_reporting(&mut self, args: &[&str]) {
        if let Err(e) = run_systemctl(args) {
            self.last_error = Some(e);
        }
    }

    fn report(&mut self, e: impl Into<KbdAnimError>) {
        self.last_error = Some(e.into().to_string());
    }
}

impl Default for GuiBackend {
    fn default() -> Self {
        GuiBackend::new()
    }
}

fn run_systemctl(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("systemctl")
        .args(args)
        .output()
        .map_err(|e| format!("could not run systemctl: {e}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(text)
}

/// Locates the real `kbdanim` CLI/daemon binary: first next to this
/// GUI binary (the normal case after `make install`), otherwise falls
/// back to `$PATH`.
fn find_kbdanim_binary() -> PathBuf {
    if let Ok(gui_exe) = std::env::current_exe() {
        if let Some(dir) = gui_exe.parent() {
            let sibling = dir.join("kbdanim");
            if sibling.exists() {
                return sibling;
            }
        }
    }
    PathBuf::from("kbdanim")
}

fn spawn_daemon() -> std::io::Result<()> {
    std::process::Command::new(find_kbdanim_binary())
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    Ok(())
}
