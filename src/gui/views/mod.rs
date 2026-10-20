//! Each file here renders exactly one tab. None of them hold state of
//! their own — all state lives on `KbdanimApp` (see `app.rs`) so a tab
//! switch never loses in-progress edits.

pub mod home;
pub mod palettes;
pub mod playlists;
pub mod presets;
pub mod settings;
