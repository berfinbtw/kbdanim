//! The Playlists tab.
//!
//! Playback does **not** spawn a second process writing to the device
//! (that would race with the daemon). Instead, pressing "Play" makes
//! sure the daemon is running and then the GUI itself advances through
//! the steps on a timer, sending an ordinary `MODE` command to the
//! daemon for each step (see `KbdanimApp::advance_playlist_if_needed`
//! in `app.rs`) — the daemon remains the single thing that ever writes
//! to the LED, exactly as with every other control in this app.

use crate::app::{KbdanimApp, PlaylistEditState};
use eframe::egui;
use std::time::Instant;

pub fn show(app: &mut KbdanimApp, ui: &mut egui::Ui) {
    ui.heading("Playlists");
    ui.add_space(6.0);

    if ui.button("New playlist").clicked() {
        app.playlist_edit = PlaylistEditState {
            open: true,
            new_step_mode: "rainbow".to_string(),
            new_step_duration: 30.0,
            ..PlaylistEditState::default()
        };
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(6.0);

    let names = app.backend.playlist_names();
    let mut play_requested: Option<String> = None;
    let mut edit_requested: Option<String> = None;
    let mut delete_requested: Option<String> = None;
    let mut stop_requested = false;

    egui::ScrollArea::vertical().show(ui, |ui| {
        if names.is_empty() {
            ui.label("No playlists yet. Create one above, or drop a .toml file in the playlists folder (see Settings > Advanced for the config path).");
        }
        for name in &names {
            ui.horizontal(|ui| {
                ui.strong(name.as_str());
                let is_playing = app.playing_playlist.as_deref() == Some(name.as_str());
                if is_playing {
                    ui.colored_label(egui::Color32::from_rgb(74, 222, 128), "▶ playing");
                    if ui.button("Stop").clicked() {
                        stop_requested = true;
                    }
                } else if ui.button("Play").clicked() {
                    play_requested = Some(name.clone());
                }
                if ui.button("Edit").clicked() {
                    edit_requested = Some(name.clone());
                }
                if ui.button("Delete").clicked() {
                    delete_requested = Some(name.clone());
                }
            });
        }
    });

    if stop_requested {
        app.playing_playlist = None;
    }
    if let Some(name) = play_requested {
        start_playlist(app, &name);
    }
    if let Some(name) = edit_requested {
        let steps = app.backend.load_playlist(&name);
        app.playlist_edit = PlaylistEditState {
            open: true,
            editing_existing: Some(name.clone()),
            name,
            steps,
            new_step_mode: "rainbow".to_string(),
            new_step_duration: 30.0,
        };
    }
    if let Some(name) = delete_requested {
        if app.playing_playlist.as_deref() == Some(name.as_str()) {
            app.playing_playlist = None;
        }
        app.backend.delete_playlist(&name);
    }

    show_editor_window(app, ui.ctx());
}

fn start_playlist(app: &mut KbdanimApp, name: &str) {
    let steps = app.backend.load_playlist(name);
    if steps.is_empty() {
        app.backend.last_error = Some(format!("Playlist '{name}' has no steps."));
        return;
    }
    let first_mode = steps[0].0.clone();
    app.playlist_steps = steps;
    app.playlist_index = 0;
    app.playlist_step_started = Instant::now();
    app.playing_playlist = Some(name.to_string());
    app.backend.set_mode(&first_mode);
}

fn show_editor_window(app: &mut KbdanimApp, ctx: &egui::Context) {
    if !app.playlist_edit.open {
        return;
    }

    let mut open = app.playlist_edit.open;
    let mut save_requested = false;

    egui::Window::new("Playlist").open(&mut open).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut app.playlist_edit.name);
        });

        ui.add_space(8.0);
        ui.label("Steps");

        let mut remove_index = None;
        let mut move_up = None;
        let mut move_down = None;
        let step_count = app.playlist_edit.steps.len();

        for i in 0..step_count {
            let (mode, duration) = app.playlist_edit.steps[i].clone();
            ui.horizontal(|ui| {
                ui.label(format!("{}. {mode} for {duration:.0}s", i + 1));
                if i > 0 && ui.small_button("↑").clicked() {
                    move_up = Some(i);
                }
                if i + 1 < step_count && ui.small_button("↓").clicked() {
                    move_down = Some(i);
                }
                if ui.small_button("Remove").clicked() {
                    remove_index = Some(i);
                }
            });
        }
        if let Some(i) = remove_index {
            app.playlist_edit.steps.remove(i);
        }
        if let Some(i) = move_up {
            app.playlist_edit.steps.swap(i, i - 1);
        }
        if let Some(i) = move_down {
            app.playlist_edit.steps.swap(i, i + 1);
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);
        ui.label("Add a step");
        ui.horizontal(|ui| {
            let animations = app.backend.animation_list();
            egui::ComboBox::from_label("Animation")
                .selected_text(app.playlist_edit.new_step_mode.clone())
                .show_ui(ui, |ui| {
                    for (name, _) in &animations {
                        ui.selectable_value(&mut app.playlist_edit.new_step_mode, name.clone(), name.clone());
                    }
                });
        });
        ui.add(
            egui::Slider::new(&mut app.playlist_edit.new_step_duration, 1.0..=600.0)
                .text("Duration (seconds)"),
        );
        if ui.button("Add step").clicked() {
            let mode = app.playlist_edit.new_step_mode.clone();
            let duration = app.playlist_edit.new_step_duration;
            app.playlist_edit.steps.push((mode, duration));
        }

        ui.add_space(10.0);
        let valid = !app.playlist_edit.name.trim().is_empty() && !app.playlist_edit.steps.is_empty();
        if !valid {
            ui.colored_label(
                egui::Color32::from_rgb(248, 113, 113),
                "Name is required, and at least one step.",
            );
        }
        ui.add_enabled_ui(valid, |ui| {
            if ui.button("Save").clicked() {
                save_requested = true;
            }
        });
    });

    if save_requested {
        let name = app.playlist_edit.name.trim().to_string();
        let steps = app.playlist_edit.steps.clone();
        let previous_name = app.playlist_edit.editing_existing.clone();

        app.backend.save_playlist(&name, &steps);
        if let Some(old_name) = previous_name {
            if old_name != name {
                app.backend.delete_playlist(&old_name);
            }
        }
        app.playlist_edit.open = false;
    } else {
        app.playlist_edit.open = open;
    }
}
