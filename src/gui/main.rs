//! `kbdanim-gui`: an optional graphical control panel for kbdanim.
//!
//! This binary only exists when built with `--features gui` (see
//! `Cargo.toml`). It never talks to hardware or runs an animation loop
//! itself — see `src/gui/backend.rs` for why, and `docs/gui.md` for the
//! full picture, including why `eframe`/`egui` was the framework
//! chosen here.

mod app;
mod backend;
mod theme;
mod views;

use app::KbdanimApp;
use eframe::egui;

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([880.0, 640.0])
            .with_min_inner_size([560.0, 420.0])
            .with_title("kbdanim"),
        ..Default::default()
    };

    eframe::run_native(
        "kbdanim",
        native_options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(KbdanimApp::new(cc)))
        }),
    )
    .expect("kbdanim-gui failed to start");
}
