# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Added
- `kbdanim-gui`: an optional graphical control panel, built only with
  `cargo build --features gui` / `make install-gui`, so the default
  CLI/daemon build stays dependency-free. Built on `eframe`/`egui`
  (see `docs/gui.md` for why).
  - Home tab: animation picker (from the live registry, nothing
    hard-coded), live color preview, color/color2 (shown only where
    relevant), brightness, speed, FPS, easing, random-mode, pause —
    every control applies to the daemon instantly, no Apply button.
  - Presets, Palettes, and Playlists tabs: full create/edit/delete/
    apply, backed by the exact same `~/.config/kbdanim/{presets,
    palettes,playlists}/*.toml` files the CLI reads and writes.
  - Settings tab: restore-state, auto power-profile (with manual
    AC/battery switching), startup animation, systemd `--user`
    autostart (no root prompt), device path, default FPS, color
    quantization.
  - Status bar: device-found / daemon-running indicators, Start/Stop,
    Turn off, and a plain-language error banner (never a panic or
    stack trace) — plus low-cost periodic polling so external CLI
    changes show up without a busy loop.
  - `src/lib.rs`: the backend is now also a library, shared verbatim
    by both `kbdanim` and `kbdanim-gui` — no second implementation of
    the color engine, animations, config, or IPC protocol for the GUI.
  - `assets/kbdanim.desktop` + `assets/kbdanim.svg`, and `make
    install-gui` / `make uninstall-gui` Makefile targets.
  - `Config::to_toml_string()` / `Config::save()`, used by the
    Settings tab (small, additive backend change; the CLI still never
    needs to write `config.toml` itself).

## [1.0.0] - Initial release

### Added
- Hardware abstraction over the Linux `leds` sysfs class
  (`brightness` + `multi_intensity`), with write-diff caching and
  optional color quantization.
- 34 real-time animations across basic/color/nature/effect/style
  categories (see `docs/animations.md`).
- Full color engine: RGB/HEX/HSV/HSL, ~90 named colors, conversions,
  hue-aware interpolation.
- Animation engine with independently configurable FPS and speed,
  drift-corrected monotonic frame pacing.
- Config, preset, palette, and playlist system (dependency-free
  TOML-subset parser).
- Power-aware profile auto-switching (AC/battery).
- Foreground `run` command and a foreground `daemon` command
  controlled over a Unix-socket IPC protocol (`stop`, `status`, `mode`,
  `color`, `brightness`, `easing`, `profile`, `random-mode`, batched
  updates).
- State persistence across daemon restarts.
- `doctor` diagnostic command, `interactive` line-based live control,
  shell completions (bash/zsh/fish), a man page, and systemd user
  service + path units.
- Unit tests across color conversion, easing, timing, config parsing,
  the animation registry, and IPC command handling, all running
  against a `MockDevice` (no real hardware required).

### Design notes
- Zero runtime dependencies (see `Cargo.toml`); a few libc functions
  are declared by hand in `src/ffi.rs` instead of pulling in `libc`.
- No double-fork daemonization; `kbdanim daemon` runs in the
  foreground and is meant to be supervised by systemd (see the note at
  the top of `src/daemon.rs`).
- `kbdanim interactive` uses line-based input rather than raw
  single-keypress `termios` control (see the doc comment on
  `cmd_interactive` in `src/main.rs` for why).
