//! The Home tab: the screen most users will live in. Every control here
//! applies to the daemon the instant it changes — there is no "Apply"
//! button (spec section 2).

use crate::app::{animation_uses_color, animation_uses_color2, color_from_rgb, KbdanimApp};
use eframe::egui;
use kbdanim::easing::Easing;

pub fn show(app: &mut KbdanimApp, ui: &mut egui::Ui) {
    if !app.status.running {
        ui.horizontal(|ui| {
            ui.label("The daemon isn't running yet — it will start automatically as soon as you change something below, or press \"Start\" above.");
        });
        ui.add_space(6.0);
    }

    ui.heading("Animation");
    let animations = app.backend.animation_list();
    let current_mode = app.status.mode.clone();
    let mut selected_mode = current_mode.clone();

    egui::ComboBox::from_label("Mode")
        .selected_text(selected_mode.clone())
        .show_ui(ui, |ui| {
            for (name, desc) in &animations {
                ui.selectable_value(&mut selected_mode, name.clone(), format!("{name} — {desc}"));
            }
        });

    if selected_mode != current_mode {
        app.backend.set_mode(&selected_mode);
        app.status.mode = selected_mode;
        app.refresh_status();
    }

    ui.add_space(10.0);

    let preview_color = if animation_uses_color(&app.status.mode) {
        egui::Color32::from_rgb(app.color[0], app.color[1], app.color[2])
    } else {
        egui::Color32::from_rgb(app.status.color.r, app.status.color.g, app.status.color.b)
    };
    let (rect, _response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().min(260.0), 56.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(rect, 6.0, preview_color);

    ui.add_space(10.0);

    if animation_uses_color(&app.status.mode) {
        ui.horizontal(|ui| {
            ui.label("Color");
            if ui.color_edit_button_srgb(&mut app.color).changed() {
                app.backend.set_color(color_from_rgb(app.color));
            }
        });
        if animation_uses_color2(&app.status.mode) {
            ui.horizontal(|ui| {
                ui.label("Color 2");
                if ui.color_edit_button_srgb(&mut app.color2).changed() {
                    app.backend.set_color2(color_from_rgb(app.color2));
                }
            });
        }
    } else {
        ui.label("This animation generates its own colors — no color controls needed.");
    }

    ui.add_space(14.0);
    ui.separator();
    ui.add_space(10.0);

    let mut brightness = app.status.brightness;
    ui.horizontal(|ui| {
        ui.label("Brightness");
        if ui.add(egui::Slider::new(&mut brightness, 0..=255)).changed() {
            app.status.brightness = brightness;
            app.backend.set_brightness(brightness);
        }
    });

    let mut speed = app.status.speed;
    ui.horizontal(|ui| {
        ui.label("Speed");
        if ui
            .add(egui::Slider::new(&mut speed, 0.1..=10.0).suffix("x"))
            .changed()
        {
            app.status.speed = speed;
            app.backend.set_speed(speed);
        }
    });

    let mut fps = app.status.fps;
    ui.horizontal(|ui| {
        ui.label("FPS");
        if ui.add(egui::Slider::new(&mut fps, 1..=120)).changed() {
            app.status.fps = fps;
            app.backend.set_fps(fps);
        }
    });

    ui.horizontal(|ui| {
        let current_easing = app.status.easing.clone();
        let mut selected_easing = current_easing.clone();
        egui::ComboBox::from_label("Easing")
            .selected_text(selected_easing.clone())
            .show_ui(ui, |ui| {
                for e in Easing::all() {
                    ui.selectable_value(&mut selected_easing, e.name().to_string(), e.name());
                }
            });
        if selected_easing != current_easing {
            app.backend.set_easing(&selected_easing);
            app.status.easing = selected_easing;
        }
    });

    ui.add_space(14.0);
    ui.separator();
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        let mut random_mode = app.status.random_mode;
        if ui
            .checkbox(&mut random_mode, "Random mode (switch animations automatically)")
            .changed()
        {
            app.status.random_mode = random_mode;
            app.backend.set_random_mode(random_mode);
        }
    });

    ui.horizontal(|ui| {
        let mut paused = app.status.paused;
        if ui.checkbox(&mut paused, "Paused").changed() {
            app.status.paused = paused;
            app.backend.set_paused(paused);
        }
    });
}
