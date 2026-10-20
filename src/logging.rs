//! A small leveled logger writing to stderr. No dependency on `log`/
//! `env_logger`: kbdanim only ever needs five levels and simple
//! `--verbose`/`--quiet` control (spec section 44).

use std::sync::atomic::{AtomicU8, Ordering};

pub const LEVEL_ERROR: u8 = 0;
pub const LEVEL_WARN: u8 = 1;
pub const LEVEL_INFO: u8 = 2;
pub const LEVEL_DEBUG: u8 = 3;
pub const LEVEL_TRACE: u8 = 4;

static CURRENT_LEVEL: AtomicU8 = AtomicU8::new(LEVEL_INFO);

pub fn set_level(level: u8) {
    CURRENT_LEVEL.store(level.min(LEVEL_TRACE), Ordering::Relaxed);
}

/// Raises the log level by `count` steps above the default (`info`),
/// used by repeated `-v`/`--verbose` flags.
pub fn set_verbose(count: u8) {
    let level = (LEVEL_INFO as u16 + count as u16).min(LEVEL_TRACE as u16) as u8;
    set_level(level);
}

/// Drops the log level to `error` only, used by `--quiet`.
pub fn set_quiet() {
    set_level(LEVEL_ERROR);
}

fn should_log(level: u8) -> bool {
    level <= CURRENT_LEVEL.load(Ordering::Relaxed)
}

pub fn error(msg: &str) {
    if should_log(LEVEL_ERROR) {
        eprintln!("[error] {msg}");
    }
}

pub fn warn(msg: &str) {
    if should_log(LEVEL_WARN) {
        eprintln!("[warn]  {msg}");
    }
}

pub fn info(msg: &str) {
    if should_log(LEVEL_INFO) {
        eprintln!("[info]  {msg}");
    }
}

pub fn debug(msg: &str) {
    if should_log(LEVEL_DEBUG) {
        eprintln!("[debug] {msg}");
    }
}

pub fn trace(msg: &str) {
    if should_log(LEVEL_TRACE) {
        eprintln!("[trace] {msg}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_verbose_increases_level() {
        set_level(LEVEL_INFO);
        set_verbose(2);
        assert_eq!(CURRENT_LEVEL.load(Ordering::Relaxed), LEVEL_TRACE.min(LEVEL_INFO + 2));
        set_level(LEVEL_INFO); // restore default for other tests
    }

    #[test]
    fn set_quiet_drops_to_error_only() {
        set_quiet();
        assert_eq!(CURRENT_LEVEL.load(Ordering::Relaxed), LEVEL_ERROR);
        set_level(LEVEL_INFO); // restore default for other tests
    }
}
