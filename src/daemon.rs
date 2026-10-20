//! The daemon process (spec section 17). Runs in the foreground by
//! design — see the module-level note below — driving the animation
//! engine, handling IPC control commands, and automatically switching
//! power profiles.
//!
//! ## Why foreground instead of a classic double-fork daemon?
//! kbdanim intentionally does *not* implement the traditional
//! fork/setsid/close-fds daemonization dance. Modern service managers
//! (systemd, the shipped `systemd/kbdanim.service` unit uses
//! `Type=simple`) already provide backgrounding, restart-on-failure,
//! logging, and lifecycle management — reimplementing that with raw
//! `fork()` would only add a large surface of subtle, hard-to-test bugs
//! for no real benefit. Users who want it backgrounded without systemd
//! can simply run `kbdanim daemon &` or use `nohup`.

use crate::animation::AnimationParams;
use crate::config::Config;
use crate::device::{KeyboardDevice, SysfsDevice};
use crate::easing::Easing;
use crate::error::Result;
use crate::ipc::{self, IpcServer};
use crate::power::PowerState;
use crate::rng::Rng;
use crate::state::{RuntimeState, SharedState};
use crate::{engine, logging, paths, signal};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub fn run(config: Config) -> Result<()> {
    paths::ensure_dirs()?;
    signal::install();

    let initial_state = if config.restore_state {
        crate::state::load_from_disk().unwrap_or_else(|| initial_state_from_config(&config))
    } else {
        initial_state_from_config(&config)
    };
    let state: SharedState = Arc::new(Mutex::new(initial_state));

    let socket_path = paths::socket_path();
    let server = IpcServer::bind(&socket_path)?;
    logging::info(&format!("kbdanim daemon listening on {}", socket_path.display()));

    let mut device = SysfsDevice::new(PathBuf::from(&config.device));
    let mut device_ok = device.exists();
    if !device_ok {
        logging::warn(&format!(
            "LED device not found at {} — waiting for it to appear",
            config.device
        ));
    }

    let (start_fps, start_brightness) = {
        let st = state.lock().unwrap();
        (st.fps, st.brightness)
    };
    let mut eng = engine::Engine::new(engine::EngineConfig {
        fps: start_fps,
        global_brightness: start_brightness,
        quantization: config.color_quantization,
    });

    let mut current_mode = {
        let st = state.lock().unwrap();
        st.mode.clone()
    };
    let mut animation = crate::animations::create(&current_mode, 0)
        .unwrap_or_else(|| crate::animations::create("rainbow", 0).expect("rainbow always exists"));

    let mut last_power_check = Instant::now();
    let mut last_power_state = PowerState::Unknown;
    let mut last_applied_profile = String::new();
    let mut last_save = Instant::now();
    let mut last_random_switch = Instant::now();
    let mut random_rng = Rng::from_entropy();
    let animation_names = crate::animations::names();

    loop {
        if signal::shutdown_requested() {
            logging::info("shutdown signal received, exiting cleanly");
            break;
        }

        if let Some(stream) = server.try_accept() {
            ipc::handle_client(stream, &state);
        }

        let random_mode_enabled = state.lock().unwrap().random_mode;
        if random_mode_enabled
            && !animation_names.is_empty()
            && last_random_switch.elapsed().as_secs_f64() >= config.random_interval
        {
            last_random_switch = Instant::now();
            let idx = random_rng.range_u32(0, (animation_names.len() - 1) as u32) as usize;
            let new_mode = animation_names[idx].to_string();
            let mut st = state.lock().unwrap();
            st.mode = new_mode.clone();
            drop(st);
            logging::debug(&format!("random-mode switched to '{new_mode}'"));
        }

        let (mode, color, color2, brightness_val, speed, fps, easing_name, paused, running, profile_name) = {
            let st = state.lock().unwrap();
            (
                st.mode.clone(),
                st.color,
                st.color2,
                st.brightness,
                st.speed,
                st.fps,
                st.easing.clone(),
                st.paused,
                st.running,
                st.profile.clone(),
            )
        };

        if !running {
            logging::info("stop requested, exiting cleanly");
            break;
        }

        if mode != current_mode {
            match crate::animations::create(&mode, 0) {
                Some(a) => {
                    animation = a;
                    current_mode = mode;
                }
                None => logging::warn(&format!("unknown mode requested: {mode}")),
            }
        }

        if config.auto_profile && last_power_check.elapsed() >= Duration::from_secs(2) {
            last_power_check = Instant::now();
            let power_state = crate::power::detect();
            if power_state != last_power_state && power_state != PowerState::Unknown {
                last_power_state = power_state;
                let target = match power_state {
                    PowerState::Ac => "ac",
                    PowerState::Battery => "battery",
                    PowerState::Unknown => "default",
                };
                apply_profile(&config, &state, target, &mut last_applied_profile);
            }
        }

        if !profile_name.is_empty() && profile_name != last_applied_profile {
            apply_profile(&config, &state, &profile_name, &mut last_applied_profile);
        }

        eng.set_fps(fps);
        eng.set_global_brightness(if paused { 0 } else { brightness_val });

        let params = AnimationParams {
            speed,
            color,
            color2,
            easing: Easing::parse(&easing_name).unwrap_or_default(),
            ..AnimationParams::default()
        };

        match eng.step(animation.as_mut(), &params, &mut device) {
            Ok(()) => {
                if !device_ok {
                    logging::info("LED device detected, resuming normal operation");
                    device.invalidate_cache();
                    device_ok = true;
                }
            }
            Err(e) => {
                if device_ok {
                    logging::warn(&format!("device write failed: {e} — waiting for device"));
                    device_ok = false;
                }
            }
        }

        if config.restore_state && last_save.elapsed() >= Duration::from_secs(5) {
            last_save = Instant::now();
            let st = state.lock().unwrap();
            let _ = crate::state::save_to_disk(&st);
        }

        eng.pace();
    }

    if config.restore_state {
        let st = state.lock().unwrap();
        let _ = crate::state::save_to_disk(&st);
    }
    let _ = device.write_brightness(0);
    logging::info("kbdanim daemon stopped");
    Ok(())
}

fn initial_state_from_config(config: &Config) -> RuntimeState {
    RuntimeState {
        mode: config.mode.clone(),
        color: config.color,
        color2: config.color2,
        brightness: config.brightness,
        speed: config.speed,
        fps: config.fps,
        easing: config.easing.clone(),
        profile: "default".to_string(),
        paused: false,
        running: true,
        random_mode: false,
    }
}

fn apply_profile(config: &Config, state: &SharedState, name: &str, last_applied: &mut String) {
    if let Some(p) = config.profiles.get(name) {
        let mut st = state.lock().unwrap();
        st.mode = p.mode.clone();
        st.brightness = p.brightness;
        st.speed = p.speed;
        st.fps = p.fps;
        st.profile = name.to_string();
        logging::info(&format!("applied profile '{name}'"));
    }
    *last_applied = name.to_string();
}
