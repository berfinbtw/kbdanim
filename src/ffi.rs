//! A handful of hand-written bindings to well-known, stable POSIX/libc
//! functions. kbdanim has zero external crate dependencies by design
//! (see Cargo.toml), so instead of pulling in the `libc` crate for two
//! functions, they are declared directly here. Every signature below
//! matches the corresponding POSIX C declaration exactly.

use std::os::raw::c_int;

extern "C" {
    #[link_name = "getuid"]
    fn c_getuid() -> u32;
}

/// The real user ID of the calling process, used to build a
/// per-user fallback runtime directory when `$XDG_RUNTIME_DIR` is
/// unset.
pub fn getuid() -> u32 {
    unsafe { c_getuid() }
}

/// `extern "C" fn(c_int)`, matching POSIX's `sighandler_t`.
pub type SigHandler = extern "C" fn(c_int);

extern "C" {
    #[link_name = "signal"]
    fn c_signal(signum: c_int, handler: SigHandler) -> SigHandler;
}

pub const SIGINT: c_int = 2;
pub const SIGTERM: c_int = 15;

/// Installs `handler` for signal `signum` via the C library's `signal()`.
///
/// # Safety-relevant notes
/// `handler` must only perform async-signal-safe operations (this
/// project's handlers only store to an `AtomicBool`, which is safe).
/// This function itself is safe to call from ordinary Rust code; the
/// `unsafe` FFI call is fully contained here.
pub fn install_signal_handler(signum: c_int, handler: SigHandler) {
    unsafe {
        c_signal(signum, handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getuid_returns_a_plausible_value() {
        // Just confirm the FFI call succeeds and returns without
        // crashing; the actual value depends on the test runner's uid.
        let _ = getuid();
    }
}
