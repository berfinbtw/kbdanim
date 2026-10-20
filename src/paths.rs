//! XDG Base Directory paths used throughout kbdanim (spec section 53).

use std::path::PathBuf;

pub fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

pub fn config_dir() -> PathBuf {
    if let Ok(v) = std::env::var("XDG_CONFIG_HOME") {
        if !v.is_empty() {
            return PathBuf::from(v).join("kbdanim");
        }
    }
    home_dir().join(".config").join("kbdanim")
}

pub fn state_dir() -> PathBuf {
    if let Ok(v) = std::env::var("XDG_STATE_HOME") {
        if !v.is_empty() {
            return PathBuf::from(v).join("kbdanim");
        }
    }
    home_dir().join(".local").join("state").join("kbdanim")
}

pub fn runtime_dir() -> PathBuf {
    if let Ok(v) = std::env::var("XDG_RUNTIME_DIR") {
        if !v.is_empty() {
            return PathBuf::from(v).join("kbdanim");
        }
    }
    std::env::temp_dir().join(format!("kbdanim-{}", crate::ffi::getuid()))
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn presets_dir() -> PathBuf {
    config_dir().join("presets")
}

pub fn palettes_dir() -> PathBuf {
    config_dir().join("palettes")
}

pub fn playlists_dir() -> PathBuf {
    config_dir().join("playlists")
}

pub fn state_file() -> PathBuf {
    state_dir().join("state.toml")
}

pub fn socket_path() -> PathBuf {
    runtime_dir().join("kbdanim.sock")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(config_dir())?;
    std::fs::create_dir_all(presets_dir())?;
    std::fs::create_dir_all(palettes_dir())?;
    std::fs::create_dir_all(playlists_dir())?;
    std::fs::create_dir_all(state_dir())?;
    std::fs::create_dir_all(runtime_dir())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_file_lives_under_config_dir() {
        assert!(config_file().starts_with(config_dir()));
    }

    #[test]
    fn state_file_lives_under_state_dir() {
        assert!(state_file().starts_with(state_dir()));
    }

    #[test]
    fn socket_path_lives_under_runtime_dir() {
        assert!(socket_path().starts_with(runtime_dir()));
    }
}
