//! The Presets tab. Reads/writes the exact same
//! `$XDG_CONFIG_HOME/kbdanim/presets/*.toml` files the CLI's
//! `kbdanim preset` subcommand uses (via `kbdanim::preset`), so a
//! preset created here shows up in `kbdanim preset list` and vice
//! versa.

use crate::app::{color_from_rgb, KbdanimApp, PresetEditState};
use eframe::egui;
use kbdanim::easing::Easing;
use kbdanim::preset::Preset;

pub fn show(app: &mut KbdanimApp, ui: &mut egui::Ui) {
    ui.heading("Presets");
    ui.add_space(6.0);

    if ui.button("New preset").clicked() {
        open_editor_for_new(app);
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(6.0);

    let presets = app.backend.presets();
    egui::ScrollArea::vertical().show(ui, |ui| {
        for p in &presets {
            let builtin = app.backend.is_builtin_preset(&p.name);
            ui.horizontal(|ui| {
                ui.strong(p.name.as_str());
                ui.label(format!(
                    "{} · brightness {} · {:.2}x · {} fps{}",
                    p.mode,
                    p.brightness,
                    p.speed,
                    p.fps,
                    if builtin { " · built-in" } else { "" }
                ));
                if ui.button("Apply").clicked() {
                    app.backend.apply_preset(&p.name);
                    app.refresh_status();
                }
                if !builtin {
                    if ui.button("Edit").clicked() {
                        open_editor_for_existing(app, p);
                    }
                    if ui.button("Delete").clicked() {
                        app.backend.delete_preset(&p.name);
                    }
                }
            });
        }
    });

    show_editor_window(app, ui.ctx());
}

fn open_editor_for_new(app: &mut KbdanimApp) {
    app.preset_edit = PresetEditState {
        open: true,
        mode: "rainbow".to_string(),
        brightness: 200,
        speed: 1.0,
        fps: 30,
        color: [0, 170, 255],
        color2: [255, 0, 128],
        easing: "smoothstep".to_string(),
        ..PresetEditState::default()
    };
}

fn open_editor_for_existing(app: &mut KbdanimApp, p: &Preset) {
    let color = p.color.map(|c| [c.r, c.g, c.b]).unwrap_or([0, 170, 255]);
    let color2 = p.color2.map(|c| [c.r, c.g, c.b]).unwrap_or([255, 0, 128]);
    app.preset_edit = PresetEditState {
        open: true,
        editing_existing: Some(p.name.clone()),
        name: p.name.clone(),
        mode: p.mode.clone(),
        brightness: p.brightness,
        speed: p.speed,
        fps: p.fps,
        use_color: p.color.is_some(),
        color,
        use_color2: p.color2.is_some(),
        color2,
        use_easing: p.easing.is_some(),
        easing: p.easing.clone().unwrap_or_else(|| "smoothstep".to_string()),
    };
}

fn show_editor_window(app: &mut KbdanimApp, ctx: &egui::Context) {
    if !app.preset_edit.open {
        return;
    }

    let mut open = app.preset_edit.open;
    let mut save_requested = false;

    egui::Window::new("Preset").open(&mut open).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut app.preset_edit.name);
        });

        let animations = app.backend.animation_list();
        egui::ComboBox::from_label("Mode")
            .selected_text(app.preset_edit.mode.clone())
            .show_ui(ui, |ui| {
                for (name, _) in &animations {
                    ui.selectable_value(&mut app.preset_edit.mode, name.clone(), name.clone());
                }
            });

        ui.add(egui::Slider::new(&mut app.preset_edit.brightness, 0..=255).text("Brightness"));
        ui.add(egui::Slider::new(&mut app.preset_edit.speed, 0.1..=10.0).text("Speed"));
        ui.add(egui::Slider::new(&mut app.preset_edit.fps, 1..=120).text("FPS"));

        ui.checkbox(&mut app.preset_edit.use_color, "Set color");
        if app.preset_edit.use_color {
            ui.color_edit_button_srgb(&mut app.preset_edit.color);
        }
        ui.checkbox(&mut app.preset_edit.use_color2, "Set color 2");
        if app.preset_edit.use_color2 {
            ui.color_edit_button_srgb(&mut app.preset_edit.color2);
        }
        ui.checkbox(&mut app.preset_edit.use_easing, "Set easing");
        if app.preset_edit.use_easing {
            egui::ComboBox::from_label("Easing")
                .selected_text(app.preset_edit.easing.clone())
                .show_ui(ui, |ui| {
                    for e in Easing::all() {
                        ui.selectable_value(&mut app.preset_edit.easing, e.name().to_string(), e.name());
                    }
                });
        }

        ui.add_space(8.0);
        let name_ok = !app.preset_edit.name.trim().is_empty();
        if !name_ok {
            ui.colored_label(egui::Color32::from_rgb(248, 113, 113), "Name is required.");
        }
        ui.add_enabled_ui(name_ok, |ui| {
            if ui.button("Save").clicked() {
                save_requested = true;
            }
        });
    });

    if save_requested {
        // Materialize everything into locals first so there is no
        // question of borrowing `app.preset_edit` across the call to
        // `app.backend.save_preset` below.
        let name = app.preset_edit.name.trim().to_string();
        let mode = app.preset_edit.mode.clone();
        let brightness = app.preset_edit.brightness;
        let speed = app.preset_edit.speed;
        let fps = app.preset_edit.fps;
        let color = app.preset_edit.use_color.then(|| color_from_rgb(app.preset_edit.color));
        let color2 = app.preset_edit.use_color2.then(|| color_from_rgb(app.preset_edit.color2));
        let easing = app.preset_edit.use_easing.then(|| app.preset_edit.easing.clone());
        let previous_name = app.preset_edit.editing_existing.clone();

        app.backend.save_preset(
            &name,
            &mode,
            brightness,
            speed,
            fps,
            color,
            color2,
            easing.as_deref(),
        );
        // If this was a rename (editing an existing preset under a new
        // name), remove the old file so we don't leave a duplicate.
        if let Some(old_name) = previous_name {
            if old_name != name {
                app.backend.delete_preset(&old_name);
            }
        }
        app.preset_edit.open = false;
    } else {
        app.preset_edit.open = open;
    }
}
