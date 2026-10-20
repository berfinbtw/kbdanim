//! A small, deliberately restrained visual theme.
//!
//! kbdanim-gui is a keyboard-lighting *controller*, not a piece of
//! "gaming RGB" chrome itself: no gradients, no blur, no glow, no
//! neon-everything. The one splash of color anywhere in the UI is the
//! live preview swatch that reflects the keyboard's actual current
//! color -- because that's information, not decoration.
//!
//! This intentionally only touches a handful of high-level, long-
//! stable `egui::Visuals` fields (base theme + a few flat colors)
//! rather than lower-level style internals (corner/margin/spacing
//! types), which have been renamed and restructured across egui
//! releases -- keeping this file simple keeps it working across a
//! wider range of egui versions.

use eframe::egui;

pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(24, 24, 27);
    visuals.window_fill = egui::Color32::from_rgb(24, 24, 27);
    visuals.extreme_bg_color = egui::Color32::from_rgb(18, 18, 20);
    visuals.selection.bg_fill = egui::Color32::from_rgb(59, 130, 246);
    ctx.set_visuals(visuals);
}
