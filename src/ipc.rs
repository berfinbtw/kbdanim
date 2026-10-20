//! A simple line-based request/response protocol over a Unix domain
//! socket (spec section 18): the client writes one command line, the
//! server writes one response, and the connection closes. No serde/
//! JSON dependency is needed for a protocol this small.

use crate::color::Color;
use crate::easing::Easing;
use crate::error::{KbdAnimError, Result};
use crate::state::{RuntimeState, SharedState};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

pub struct IpcServer {
    listener: UnixListener,
    path: PathBuf,
}

impl IpcServer {
    /// Binds a fresh listening socket at `path`, removing any stale
    /// socket file left behind by a previous crashed run, and restricts
    /// its permissions to the owning user only (spec section 65).
    pub fn bind(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        let listener = UnixListener::bind(path)
            .map_err(|e| KbdAnimError::SocketError(format!("{}: {}", path.display(), e)))?;
        if let Ok(meta) = std::fs::metadata(path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o700);
            let _ = std::fs::set_permissions(path, perms);
        }
        listener
            .set_nonblocking(true)
            .map_err(|e| KbdAnimError::SocketError(e.to_string()))?;
        Ok(IpcServer {
            listener,
            path: path.to_path_buf(),
        })
    }

    /// Non-blocking accept: returns `Some(stream)` if a client is
    /// already waiting, or `None` immediately otherwise. Called from the
    /// daemon's main loop so IPC handling never stalls frame pacing.
    pub fn try_accept(&self) -> Option<UnixStream> {
        self.listener.accept().ok().map(|(stream, _addr)| stream)
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Handles one already-accepted client connection: reads a single
/// command line, mutates `state` accordingly, and writes back a single
/// response line.
pub fn handle_client(mut stream: UnixStream, state: &SharedState) {
    let read_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let _ = read_stream.set_nonblocking(false);
    let mut reader = BufReader::new(read_stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let response = process_command(line.trim(), state);
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.write_all(b"\n");
}

/// Sends a single command to a running daemon and returns its response.
pub fn send_command(path: &Path, cmd: &str) -> Result<String> {
    let mut stream = UnixStream::connect(path).map_err(|e| {
        KbdAnimError::SocketError(format!(
            "could not connect to daemon at {}: {e} (is the daemon running? try 'kbdanim daemon')",
            path.display()
        ))
    })?;
    stream.write_all(cmd.as_bytes())?;
    stream.write_all(b"\n")?;
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response.trim().to_string())
}

fn process_command(cmd: &str, state: &SharedState) -> String {
    let mut parts = cmd.splitn(2, ' ');
    let verb = parts.next().unwrap_or("").to_uppercase();
    let arg = parts.next().unwrap_or("").trim();

    let mut st = match state.lock() {
        Ok(g) => g,
        Err(_) => return "ERR state lock poisoned".to_string(),
    };

    match verb.as_str() {
        "PING" => "OK pong".to_string(),
        "STATUS" => format_status(&st),
        "STOP" => {
            st.running = false;
            "OK stopping".to_string()
        }
        "MODE" => {
            if crate::animations::create(arg, 0).is_some() {
                st.mode = arg.to_string();
                "OK mode set".to_string()
            } else {
                format!("ERR unknown mode: {arg}")
            }
        }
        "BRIGHTNESS" => match arg.parse::<i32>() {
            Ok(v) if (0..=255).contains(&v) => {
                st.brightness = v as u8;
                "OK brightness set".to_string()
            }
            _ => "ERR brightness must be 0-255".to_string(),
        },
        "SPEED" => match arg.parse::<f64>() {
            Ok(v) if (0.1..=10.0).contains(&v) => {
                st.speed = v;
                "OK speed set".to_string()
            }
            _ => "ERR speed must be 0.1-10.0".to_string(),
        },
        "FPS" => match arg.parse::<u32>() {
            Ok(v) if (1..=120).contains(&v) => {
                st.fps = v;
                "OK fps set".to_string()
            }
            _ => "ERR fps must be 1-120".to_string(),
        },
        "COLOR" => match Color::parse(arg) {
            Ok(c) => {
                st.color = c;
                "OK color set".to_string()
            }
            Err(e) => format!("ERR {e}"),
        },
        "COLOR2" => match Color::parse(arg) {
            Ok(c) => {
                st.color2 = c;
                "OK color2 set".to_string()
            }
            Err(e) => format!("ERR {e}"),
        },
        "EASING" => match Easing::parse(arg) {
            Some(_) => {
                st.easing = arg.to_lowercase();
                "OK easing set".to_string()
            }
            None => format!("ERR unknown easing: {arg}"),
        },
        "PROFILE" => {
            st.profile = arg.to_string();
            "OK profile switch requested".to_string()
        }
        "RANDOM_MODE" => {
            let on = matches!(arg.to_uppercase().as_str(), "ON" | "TRUE" | "1");
            let off = matches!(arg.to_uppercase().as_str(), "OFF" | "FALSE" | "0");
            if on {
                st.random_mode = true;
                "OK random-mode enabled".to_string()
            } else if off {
                st.random_mode = false;
                "OK random-mode disabled".to_string()
            } else {
                "ERR expected 'on' or 'off'".to_string()
            }
        }
        "PAUSE" => {
            st.paused = true;
            "OK paused".to_string()
        }
        "RESUME" => {
            st.paused = false;
            "OK resumed".to_string()
        }
        "BATCH" => {
            for kv in arg.split(';') {
                let kv = kv.trim();
                if kv.is_empty() {
                    continue;
                }
                if let Some((k, v)) = kv.split_once('=') {
                    apply_single_field(&mut st, k.trim(), v.trim());
                }
            }
            "OK batch applied".to_string()
        }
        "" => "ERR empty command".to_string(),
        other => format!("ERR unknown command: {other}"),
    }
}

fn apply_single_field(st: &mut RuntimeState, key: &str, value: &str) {
    match key {
        "mode" => {
            if crate::animations::create(value, 0).is_some() {
                st.mode = value.to_string();
            }
        }
        "brightness" => {
            if let Ok(v) = value.parse::<i32>() {
                if (0..=255).contains(&v) {
                    st.brightness = v as u8;
                }
            }
        }
        "speed" => {
            if let Ok(v) = value.parse::<f64>() {
                st.speed = v;
            }
        }
        "fps" => {
            if let Ok(v) = value.parse::<u32>() {
                st.fps = v.max(1);
            }
        }
        "color" => {
            if let Ok(c) = Color::parse(value) {
                st.color = c;
            }
        }
        "color2" => {
            if let Ok(c) = Color::parse(value) {
                st.color2 = c;
            }
        }
        "easing" => {
            if Easing::parse(value).is_some() {
                st.easing = value.to_lowercase();
            }
        }
        "profile" => {
            st.profile = value.to_string();
        }
        _ => {}
    }
}

fn format_status(st: &RuntimeState) -> String {
    format!(
        "OK\nmode={}\nbrightness={}\nspeed={}\nfps={}\ncolor={}\ncolor2={}\neasing={}\nprofile={}\npaused={}\nrandom_mode={}",
        st.mode,
        st.brightness,
        st.speed,
        st.fps,
        st.color.to_hex(),
        st.color2.to_hex(),
        st.easing,
        st.profile,
        st.paused,
        st.random_mode,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn fresh_state() -> SharedState {
        Arc::new(Mutex::new(RuntimeState::default()))
    }

    #[test]
    fn ping_returns_pong() {
        let state = fresh_state();
        assert_eq!(process_command("PING", &state), "OK pong");
    }

    #[test]
    fn mode_accepts_known_animation() {
        let state = fresh_state();
        let resp = process_command("MODE fire", &state);
        assert!(resp.starts_with("OK"));
        assert_eq!(state.lock().unwrap().mode, "fire");
    }

    #[test]
    fn mode_rejects_unknown_animation() {
        let state = fresh_state();
        let resp = process_command("MODE not-a-real-mode", &state);
        assert!(resp.starts_with("ERR"));
    }

    #[test]
    fn brightness_out_of_range_is_rejected() {
        let state = fresh_state();
        let resp = process_command("BRIGHTNESS 999", &state);
        assert!(resp.starts_with("ERR"));
    }

    #[test]
    fn stop_sets_running_false() {
        let state = fresh_state();
        process_command("STOP", &state);
        assert!(!state.lock().unwrap().running);
    }

    #[test]
    fn batch_applies_multiple_fields() {
        let state = fresh_state();
        let resp = process_command("BATCH mode=fire;brightness=123;speed=1.5", &state);
        assert!(resp.starts_with("OK"));
        let st = state.lock().unwrap();
        assert_eq!(st.mode, "fire");
        assert_eq!(st.brightness, 123);
        assert_eq!(st.speed, 1.5);
    }

    #[test]
    fn unknown_command_is_error() {
        let state = fresh_state();
        assert!(process_command("NONSENSE", &state).starts_with("ERR"));
    }
}
