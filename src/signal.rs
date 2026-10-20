//! Installs handlers for SIGINT/SIGTERM so the daemon (and the
//! foreground `run` command) can shut down cleanly: stop the animation
//! loop, persist state, turn the LED off, then exit (spec section 66).

use crate::ffi::{self, SIGINT, SIGTERM};
use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, Ordering};

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_shutdown_signal(_signum: c_int) {
    // Async-signal-safe: this only stores to an atomic, nothing else.
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

/// Installs handlers for SIGINT and SIGTERM. Call once, early in `main`.
pub fn install() {
    ffi::install_signal_handler(SIGINT, handle_shutdown_signal);
    ffi::install_signal_handler(SIGTERM, handle_shutdown_signal);
}

pub fn shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}

/// Clears the shutdown flag. Mainly useful in tests that install
/// handlers repeatedly within the same process.
pub fn reset() {
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_out_not_requested() {
        reset();
        assert!(!shutdown_requested());
    }

    #[test]
    fn handler_sets_the_flag() {
        reset();
        handle_shutdown_signal(0);
        assert!(shutdown_requested());
        reset();
    }
}
