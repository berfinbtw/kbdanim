//! The main `eframe::App` implementation: overall window layout (status
//! bar, tab selector, error banner) plus all state the individual tab
//! views (`views/*.rs`) read and mutate. The views hold no state of
//! their own beyond what's declared here, to keep this file the single
//! source of truth for "what can the GUI show".

use crate::backend::{DaemonStatus, GuiBackend};
use crate::views;
use eframe::egui;
use kbdanim::color::Color;
use std::time::{Duration, Instant};

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Home,
    Presets,
    Palettes,
    Playlists,
    Settings,
}

/// Animations whose look depends on `color` (spec section 6). Any
/// animation not listed here generates its own palette internally and
/// the color pickers are disabled for it, matching the CLI's own
/// `default_colors_for` split of "themed" vs "flexible" animations
/// (see `src/animation.rs`).
const USES_COLOR: &[&str] = &[
    "static",
    "breathing",
    "pulse",
    "heartbeat",
    "strobe",
    "lightning",
    "sparkle",
    "color-cycle",
    "oscillation",
    "wave",
    "meteor",
    "cyberpunk",
    "police",
];

/// Animations that additionally use a second color.
const USES_COLOR2: &[&str] = &["color-cycle", "oscillation", "wave", "meteor", "cyberpunk", "police"];

pub fn animation_uses_color(name: &str) -> bool {
    USES_COLOR.contains(&name)
}

pub fn animation_uses_color2(name: &str) -> bool {
    USES_COLOR2.contains(&name)
}

#[derive(Default)]
pub struct PresetEditState {
    pub open: bool,
    pub editing_existing: Option<String>,
    pub name: String,
    pub mode: String,
    pub brightness: u8,
    pub speed: f64,
    pub fps: u32,
    pub use_color: bool,
    pub color: [u8; 3],
    pub use_color2: bool,
    pub color2: [u8; 3],
    pub use_easing: bool,
    pub easing: String,
}

#[derive(Default)]
pub struct PaletteEditState {
    pub open: bool,
    pub editing_existing: Option<String>,
    pub name: String,
    pub colors: Vec<[u8; 3]>,
}

#[derive(Default)]
pub struct PlaylistEditState {
    pub open: bool,
    pub editing_existing: Option<String>,
    pub name: String,
    pub steps: Vec<(String, f64)>,
    pub new_step_mode: String,
    pub new_step_duration: f64,
}

pub struct KbdanimApp {
    pub backend: GuiBackend,
    pub status: DaemonStatus,
    pub tab: Tab,
    pub last_poll: Instant,

    pub color: [u8; 3],
    pub color2: [u8; 3],

    pub preset_edit: PresetEditState,
    pub palette_edit: PaletteEditState,
    pub playlist_edit: PlaylistEditState,

    // Simple, GUI-driven playlist playback: advances by sending BATCH
    // commands to the (already running) daemon on a timer, rather than
    // running a second animation loop (see docs/gui.md).
    pub playing_playlist: Option<String>,
    pub playlist_steps: Vec<(String, f64)>,
    pub playlist_index: usize,
    pub playlist_step_started: Instant,
}

impl KbdanimApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut backend = GuiBackend::new();
        let status = backend.fetch_status();
        let color = [status.color.r, status.color.g, status.color.b];
        let color2 = [status.color2.r, status.color2.g, status.color2.b];
        KbdanimApp {
            backend,
            status,
            tab: Tab::Home,
            last_poll: Instant::now(),
            color,
            color2,
            preset_edit: PresetEditState::default(),
            palette_edit: PaletteEditState::default(),
            playlist_edit: PlaylistEditState::default(),
            playing_playlist: None,
            playlist_steps: Vec::new(),
            playlist_index: 0,
            playlist_step_started: Instant::now(),
        }
    }

    /// Re-reads daemon status into `self.status` and syncs the local
    /// color-picker buffers, but only when the picker isn't currently
    /// mid-drag-looking-different from the daemon (kept simple: we
    /// just always sync — a local slider always wins for the instant
    /// the user is touching it because eframe delivers that same-frame
    /// change immediately afterwards).
    pub fn refresh_status(&mut self) {
        self.status = self.backend.fetch_status();
        if self.status.running {
            self.color = [self.status.color.r, self.status.color.g, self.status.color.b];
            self.color2 = [self.status.color2.r, self.status.color2.g, self.status.color2.b];
        }
    }

    fn advance_playlist_if_needed(&mut self) {
        if self.playing_playlist.is_none() || self.playlist_steps.is_empty() {
            return;
        }
        let (_, duration) = self.playlist_steps[self.playlist_index];
        if self.playlist_step_started.elapsed() >= Duration::from_secs_f64(duration.max(0.5)) {
            self.playlist_index = (self.playlist_index + 1) % self.playlist_steps.len();
            self.playlist_step_started = Instant::now();
            let mode = self.playlist_steps[self.playlist_index].0.clone();
            self.backend.set_mode(&mode);
        }
    }
}

impl eframe::App for KbdanimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Low-cost periodic poll (spec section 22): catches changes
        // made from another terminal via the CLI, without a busy loop.
        if self.last_poll.elapsed() >= Duration::from_millis(700) {
            self.last_poll = Instant::now();
            self.refresh_status();
        }
        self.advance_playlist_if_needed();
        ctx.request_repaint_after(Duration::from_millis(300));

        egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("kbdanim");
                ui.separator();

                let device_ok = self.backend.device_present();
                let (dot, label) = if device_ok {
                    ("●", "Keyboard connected")
                } else {
                    ("●", "Keyboard not found")
                };
                ui.colored_label(
                    if device_ok {
                        egui::Color32::from_rgb(74, 222, 128)
                    } else {
                        egui::Color32::from_rgb(248, 113, 113)
                    },
                    dot,
                );
                ui.label(label);

                ui.separator();

                let (dot_color, daemon_label) = if self.status.running {
                    (egui::Color32::from_rgb(74, 222, 128), "Daemon running")
                } else {
                    (egui::Color32::from_rgb(161, 161, 170), "Daemon stopped")
                };
                ui.colored_label(dot_color, "●");
                ui.label(daemon_label);

                if self.status.running {
                    if ui.button("Stop").clicked() {
                        self.backend.stop_daemon();
                        self.playing_playlist = None;
                        self.refresh_status();
                    }
                } else if ui.button("Start").clicked() {
                    self.backend.ensure_daemon_running();
                    self.refresh_status();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Turn off").clicked() {
                        self.backend.turn_off();
                        self.refresh_status();
                    }
                });
            });
            ui.add_space(4.0);
        });

        if let Some(err) = self.backend.last_error.clone() {
            egui::TopBottomPanel::top("error_banner").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(248, 113, 113), "⚠");
                    ui.label(err.as_str());
                    if ui.small_button("Dismiss").clicked() {
                        self.backend.last_error = None;
                    }
                });
            });
        }

        egui::SidePanel::left("tabs").resizable(false).show(ctx, |ui| {
            ui.add_space(6.0);
            ui.selectable_value(&mut self.tab, Tab::Home, "Home");
            ui.selectable_value(&mut self.tab, Tab::Presets, "Presets");
            ui.selectable_value(&mut self.tab, Tab::Palettes, "Palettes");
            ui.selectable_value(&mut self.tab, Tab::Playlists, "Playlists");
            ui.selectable_value(&mut self.tab, Tab::Settings, "Settings");
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Home => views::home::show(self, ui),
            Tab::Presets => views::presets::show(self, ui),
            Tab::Palettes => views::palettes::show(self, ui),
            Tab::Playlists => views::playlists::show(self, ui),
            Tab::Settings => views::settings::show(self, ui),
        });
    }
}

pub fn color_from_rgb(rgb: [u8; 3]) -> Color {
    Color::new(rgb[0], rgb[1], rgb[2])
}
