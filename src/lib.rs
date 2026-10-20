//! kbdanim's backend as a library.
//!
//! `src/main.rs` (the `kbdanim` CLI/daemon binary) and `src/gui/main.rs`
//! (the optional `kbdanim-gui` binary, built with `--features gui`)
//! both depend on this library and nothing else talks to hardware,
//! config files, or the daemon socket. There is exactly one backend.

pub mod animation;
pub mod animations;
pub mod brightness;
pub mod cli;
pub mod color;
pub mod config;
pub mod daemon;
pub mod device;
pub mod easing;
pub mod engine;
pub mod error;
pub mod ffi;
pub mod ipc;
pub mod logging;
pub mod palette;
pub mod paths;
pub mod power;
pub mod preset;
pub mod profile;
pub mod rng;
pub mod signal;
pub mod state;
pub mod sysfs;
pub mod timing;
pub mod toml_lite;
