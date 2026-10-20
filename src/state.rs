//! The daemon's live, mutable runtime state (spec section 40), shared
//! between the animation loop and the IPC command handler via a mutex,
//! and periodically persisted to disk so a restart resumes where it
//! left off (spec sections 25 and 40).

use crate::color::Color;
use crate::error::Result;
use crate::paths;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct RuntimeState {
    pub mode: String,
    pub color: Color,
    pub color2: Color,
    pub brightness: u8,
    pub speed: f64,
    pub fps: u32,
    pub easing: String,
    pub profile: String,
    pub paused: bool,
    pub running: bool,
    /// When true, the daemon periodically switches to a random
    /// animation on its own (spec section 37).
    pub random_mode: bool,
}

impl Default for RuntimeState {
    fn default() -> Self {
        RuntimeState {
            mode: "rainbow".to_string(),
            color: Color::new(0, 170, 255),
            color2: Color::new(255, 0, 128),
            brightness: 200,
            speed: 1.0,
            fps: 30,
            easing: "smoothstep".to_string(),
            profile: "default".to_string(),
            paused: false,
            running: true,
            random_mode: false,
        }
    }
}

pub type SharedState = Arc<Mutex<RuntimeState>>;

pub fn save_to_disk(state: &RuntimeState) -> Result<()> {
    paths::ensure_dirs()?;
    let text = format!(
        "mode = \"{}\"\ncolor = \"{}\"\ncolor2 = \"{}\"\nbrightness = {}\nspeed = {}\nfps = {}\neasing = \"{}\"\nprofile = \"{}\"\nrandom_mode = {}\n",
        state.mode,
        state.color.to_hex(),
        state.color2.to_hex(),
        state.brightness,
        state.speed,
        state.fps,
        state.easing,
        state.profile,
        state.random_mode,
    );
    std::fs::write(paths::state_file(), text)?;
    Ok(())
}

pub fn load_from_disk() -> Option<RuntimeState> {
    let text = std::fs::read_to_string(paths::state_file()).ok()?;
    let doc = crate::toml_lite::parse(&text).ok()?;
    let mut st = RuntimeState::default();
    if let Some(v) = doc.root.get_str("mode") {
        st.mode = v.to_string();
    }
    if let Some(v) = doc.root.get_str("color") {
        if let Ok(c) = Color::parse(v) {
            st.color = c;
        }
    }
    if let Some(v) = doc.root.get_str("color2") {
        if let Ok(c) = Color::parse(v) {
            st.color2 = c;
        }
    }
    if let Some(v) = doc.root.get_i64("brightness") {
        st.brightness = v.clamp(0, 255) as u8;
    }
    if let Some(v) = doc.root.get_f64("speed") {
        st.speed = v;
    }
    if let Some(v) = doc.root.get_i64("fps") {
        st.fps = v.max(1) as u32;
    }
    if let Some(v) = doc.root.get_str("easing") {
        st.easing = v.to_string();
    }
    if let Some(v) = doc.root.get_str("profile") {
        st.profile = v.to_string();
    }
    if let Some(v) = doc.root.get_bool("random_mode") {
        st.random_mode = v;
    }
    st.running = true;
    Some(st)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_running_and_unpaused() {
        let st = RuntimeState::default();
        assert!(st.running);
        assert!(!st.paused);
    }
}
