//! The Palettes tab. Built-in palettes are shown but read-only; user
//! palettes are the same `$XDG_CONFIG_HOME/kbdanim/palettes/*.toml`
//! files `--palette <name>` reads on the CLI.

use crate::app::{KbdanimApp, PaletteEditState};
use eframe::egui;
use kbdanim::color::Color;

pub fn show(app: &mut KbdanimApp, ui: &mut egui::Ui) {
    ui.heading("Palettes");
    ui.add_space(6.0);

    if ui.button("New palette").clicked() {
        app.palette_edit = PaletteEditState {
            open: true,
            colors: vec![[0, 170, 255], [255, 0, 128]],
            ..PaletteEditState::default()
        };
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(6.0);

    let palettes = app.backend.palettes();
    let mut delete_requested: Option<String> = None;
    let mut edit_requested: Option<(String, Vec<[u8; 3]>)> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (name, colors, is_user) in &palettes {
            ui.horizontal(|ui| {
                ui.strong(name.as_str());
                for c in colors {
                    let (rect, _response) =
                        ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(rect, 3.0, egui::Color32::from_rgb(c.r, c.g, c.b));
                }
                if *is_user {
                    if ui.button("Edit").clicked() {
                        edit_requested =
                            Some((name.clone(), colors.iter().map(|c| [c.r, c.g, c.b]).collect()));
                    }
                    if ui.button("Delete").clicked() {
                        delete_requested = Some(name.clone());
                    }
                } else {
                    ui.label("built-in");
                }
            });
        }
    });

    if let Some(name) = delete_requested {
        app.backend.delete_palette(&name);
    }
    if let Some((name, colors)) = edit_requested {
        app.palette_edit = PaletteEditState {
            open: true,
            editing_existing: Some(name.clone()),
            name,
            colors,
        };
    }

    show_editor_window(app, ui.ctx());
}

fn show_editor_window(app: &mut KbdanimApp, ctx: &egui::Context) {
    if !app.palette_edit.open {
        return;
    }

    let mut open = app.palette_edit.open;
    let mut save_requested = false;

    egui::Window::new("Palette").open(&mut open).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut app.palette_edit.name);
        });

        ui.add_space(6.0);
        let mut remove_index = None;
        for (i, color) in app.palette_edit.colors.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.color_edit_button_srgb(color);
                if ui.small_button("Remove").clicked() {
                    remove_index = Some(i);
                }
            });
        }
        if let Some(i) = remove_index {
            app.palette_edit.colors.remove(i);
        }

        if ui.button("Add color").clicked() {
            app.palette_edit.colors.push([255, 255, 255]);
        }

        ui.add_space(8.0);
        let valid = !app.palette_edit.name.trim().is_empty() && app.palette_edit.colors.len() >= 2;
        if !valid {
            ui.colored_label(
                egui::Color32::from_rgb(248, 113, 113),
                "Name is required, and at least 2 colors.",
            );
        }
        ui.add_enabled_ui(valid, |ui| {
            if ui.button("Save").clicked() {
                save_requested = true;
            }
        });
    });

    if save_requested {
        let name = app.palette_edit.name.trim().to_string();
        let colors: Vec<Color> = app
            .palette_edit
            .colors
            .iter()
            .map(|c| Color::new(c[0], c[1], c[2]))
            .collect();
        let previous_name = app.palette_edit.editing_existing.clone();

        app.backend.save_palette(&name, &colors);
        if let Some(old_name) = previous_name {
            if old_name != name {
                app.backend.delete_palette(&old_name);
            }
        }
        app.palette_edit.open = false;
    } else {
        app.palette_edit.open = open;
    }
}
