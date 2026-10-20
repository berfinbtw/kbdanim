//! The Settings tab. Unlike Home, most of these are `config.toml`
//! values the daemon only reads at startup (spec sections 12-14, 20-21)
//! — the GUI writes them immediately via `Config::save`, but is honest
//! in the UI that they take effect on the next daemon (re)start, with a
//! one-click way to do that right there.

use crate::app::KbdanimApp;
use eframe::egui;
use std::time::Duration;

pub fn show(app: &mut KbdanimApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("General");
        ui.add_space(6.0);

        let mut restore_state = app.backend.config.restore_state;
        if ui
            .checkbox(&mut restore_state, "Restore previous state when the daemon starts")
            .changed()
        {
            app.backend.config.restore_state = restore_state;
            app.backend.save_config();
        }

        let mut auto_profile = app.backend.config.auto_profile;
        if ui
            .checkbox(&mut auto_profile, "Automatically switch profile on AC/battery change")
            .changed()
        {
            app.backend.config.auto_profile = auto_profile;
            app.backend.save_config();
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let animations = app.backend.animation_list();
            let mut selected = app.backend.config.mode.clone();
            egui::ComboBox::from_label("Startup animation")
                .selected_text(selected.clone())
                .show_ui(ui, |ui| {
                    for (name, _) in &animations {
                        ui.selectable_value(&mut selected, name.clone(), name.clone());
                    }
                });
            if selected != app.backend.config.mode {
                app.backend.config.mode = selected;
                app.backend.save_config();
            }
        });

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);
        ui.heading("Autostart");
        ui.label("Uses your user systemd instance — no root password needed.");
        ui.add_space(4.0);

        let enabled = app.backend.systemd_is_enabled();
        let active = app.backend.systemd_is_active();
        let status_text = if active {
            "Service is running"
        } else if enabled {
            "Enabled, but not running right now"
        } else {
            "Not installed/enabled (see README for `systemctl --user enable`)"
        };
        ui.label(status_text);

        ui.horizontal(|ui| {
            let mut enabled_toggle = enabled;
            if ui.checkbox(&mut enabled_toggle, "Start automatically at login").changed() {
                if enabled_toggle {
                    app.backend.systemd_enable();
                } else {
                    app.backend.systemd_disable();
                }
            }
            if active {
                if ui.button("Stop service").clicked() {
                    app.backend.systemd_stop();
                }
            } else if ui.button("Start service now").clicked() {
                app.backend.systemd_start();
            }
        });

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);
        ui.heading("Power");

        let power_text = match app.backend.power_state() {
            kbdanim::power::PowerState::Ac => "On AC power",
            kbdanim::power::PowerState::Battery => "On battery",
            kbdanim::power::PowerState::Unknown => "Power source unknown",
        };
        ui.label(power_text);
        if app.status.running {
            ui.label(format!("Current profile: {}", app.status.profile));
        }
        ui.horizontal(|ui| {
            if ui.button("Switch to AC profile now").clicked() {
                app.backend.switch_profile("ac");
            }
            if ui.button("Switch to battery profile now").clicked() {
                app.backend.switch_profile("battery");
            }
        });

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);
        ui.heading("Device");

        ui.horizontal(|ui| {
            ui.label("Device path");
            let mut device_path = app.backend.config.device.clone();
            let response = ui.text_edit_singleline(&mut device_path);
            if response.lost_focus() && device_path != app.backend.config.device {
                app.backend.config.device = device_path;
                app.backend.save_config();
            }
        });

        let present = app.backend.device_present();
        let writable = app.backend.device_writable();
        let device_status = if !present {
            "Not found at this path."
        } else if writable {
            "Found and writable."
        } else {
            "Found, but not writable — see the udev setup in the README."
        };
        ui.label(device_status);

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);
        ui.heading("Performance");

        let mut default_fps = app.backend.config.fps;
        ui.horizontal(|ui| {
            ui.label("Default FPS");
            if ui.add(egui::Slider::new(&mut default_fps, 1..=120)).changed() {
                app.backend.config.fps = default_fps;
                app.backend.save_config();
            }
        });

        let mut quantization = app.backend.config.color_quantization as u32;
        ui.horizontal(|ui| {
            ui.label("Color quantization");
            if ui.add(egui::Slider::new(&mut quantization, 1..=32)).changed() {
                app.backend.config.color_quantization = quantization as u8;
                app.backend.save_config();
            }
        });

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);
        ui.heading("Advanced");

        ui.label(format!("Config file: {}", app.backend.config_path().display()));
        if ui.button("Reload config from disk").clicked() {
            app.backend.reload_config();
        }

        ui.add_space(6.0);
        ui.label(
            "The settings above take effect the next time the daemon (re)starts, not instantly.",
        );
        if app.status.running && ui.button("Restart daemon now to apply").clicked() {
            app.backend.stop_daemon();
            std::thread::sleep(Duration::from_millis(200));
            app.backend.reload_config();
            app.backend.ensure_daemon_running();
            app.refresh_status();
        }
    });
}
