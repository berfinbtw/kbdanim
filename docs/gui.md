# kbdanim-gui

A minimal graphical control panel for kbdanim, for people who'd rather
not use the terminal. It's a full control surface for the same backend
the CLI uses — not a second, simpler implementation.

## Why eframe/egui?

kbdanim-gui is built with [`eframe`](https://crates.io/crates/eframe)
(the application shell for [`egui`](https://crates.io/crates/egui)),
chosen over GTK4/libadwaita or Iced for a few concrete reasons:

- **Pure Rust, no system toolkit dependency.** Building `kbdanim-gui`
  needs no `-dev` packages installed beyond what Rust/Cargo already
  need (unlike `gtk4-rs`, which requires the GTK4 development headers
  and pkg-config to be present at build time). This matters for a
  small utility people build from source.
- **Works the same on X11 and Wayland** through `winit`, with no
  extra configuration.
- **Immediate-mode fits this app well.** Every control in kbdanim-gui
  is "read the daemon's current value, draw a widget, if it changed
  write it back" — which is exactly the shape egui is built around, so
  the UI code stays simple and easy to audit.
- **Low resource use.** No embedded web view, no separate renderer
  process.

The trade-off is a UI that looks like "a clean Rust app" rather than a
pixel-perfect GNOME/GTK citizen. Given the explicit design goal of
staying minimal and out of the way rather than chasing a native look,
that trade-off was judged worth it. If you'd rather have a GTK4/
libadwaita frontend, the service layer in `src/gui/backend.rs` is
UI-toolkit-agnostic on purpose — it should be straightforward to build
an alternative frontend against it without touching the daemon, IPC
protocol, or animation engine at all.

## Building and installing

The GUI is entirely opt-in and never built by default:

```bash
# CLI/daemon only (default, zero dependencies):
cargo build --release

# CLI/daemon + GUI:
make install          # installs kbdanim (the CLI/daemon)
make install-gui      # installs kbdanim-gui, its .desktop entry, and its icon
```

Or manually:

```bash
cargo build --release --features gui --bin kbdanim-gui
```

`kbdanim-gui` looks for the real `kbdanim` binary next to itself first
(the normal case after `make install-gui`), falling back to `$PATH`.
It needs `kbdanim` to be installed for anything to actually happen —
the GUI never talks to hardware directly (see "Architecture" below).

## Running

```bash
kbdanim-gui
```

Or launch "kbdanim" from your application menu after `make
install-gui`.

## What you can do from the GUI

- **Home** — pick an animation (from the same registry `kbdanim list`
  uses — nothing is hard-coded), see a live color preview, and adjust
  color/color2 (only shown for animations that use them), brightness,
  speed, FPS, and easing. Every change applies immediately; there is
  no Apply button. Random-mode and pause are here too.
- **Presets** — list, apply, create, edit, and delete presets. These
  are the exact same `~/.config/kbdanim/presets/*.toml` files
  `kbdanim preset` uses.
- **Palettes** — view built-in palettes (read-only) and your own
  (editable) as color swatches; create, edit, delete.
- **Playlists** — build a sequence of animation/duration steps, reorder
  them, and play/stop. Playback works by having the GUI send the
  daemon a `MODE` command for each step on a timer — it never starts a
  second process writing to the device (see "Architecture").
- **Settings** — restore-state, auto power-profile, startup animation,
  systemd autostart (`systemctl --user`, no root needed), device path,
  default FPS, color quantization, and manual AC/battery profile
  switching. These map to `config.toml` and take effect the next time
  the daemon (re)starts; the page says so and offers a one-click
  restart.

The top bar always shows whether the keyboard device was found and
whether the daemon is running, with Start/Stop and a "Turn off"
shortcut. If something goes wrong (device missing, permission denied,
daemon unreachable), a plain-language banner explains what happened —
never a Rust panic or stack trace.

## Architecture

```
kbdanim-gui (src/gui/)
   main.rs      window setup, eframe::run_native
   app.rs       KbdanimApp: all UI state, tab layout, top bar, polling
   backend.rs   the ONLY place that talks to the rest of kbdanim:
                daemon IPC, config.toml, presets, palettes, playlists,
                power state, systemd --user
   views/*.rs   one file per tab, pure UI code over `backend.rs`
        |
        v
kbdanim (library, src/lib.rs)      <-- the exact same code the CLI uses
   ipc, config, preset, palette, paths, animations, power, device, ...
        |
        v
kbdanim daemon (separate process, started via `kbdanim daemon`)
   the only thing that ever writes to /sys/class/leds/.../{brightness,multi_intensity}
```

The GUI never opens a sysfs file and never runs an animation loop. It
either reads config/preset/palette/playlist files through the same
library functions the CLI uses, or sends the exact same Unix-socket
commands (`MODE`, `COLOR`, `BRIGHTNESS`, `BATCH`, ...) the CLI's
`kbdanim mode`/`color`/`brightness`/etc. send. If the daemon isn't
running when you touch a control, the GUI starts it the same way
`kbdanim interactive` does — by spawning the real `kbdanim daemon`
process, never a second implementation of it.

Because of this, the GUI and CLI can be used interchangeably at the
same time: run `kbdanim status` in a terminal while kbdanim-gui is
open and you'll see the same state (the GUI polls for external changes
about twice a second).

## Troubleshooting

- **"Keyboard not found"** — your LED device isn't at
  `/sys/class/leds/rgb:kbd_backlight`. Set the correct path in Settings
  > Device, or edit `device` in `config.toml`.
- **Device found, but not writable** — see the udev setup in the main
  README (`udev/60-kbdanim.rules`); this is the same requirement as
  running the CLI without `sudo`.
- **"Daemon stopped" and nothing responds when you move a slider** —
  the GUI should auto-start the daemon on your first change; if it
  can't, the error banner will say why (most commonly: `kbdanim` isn't
  installed/on `$PATH`, or the udev permission above isn't set up).
- **Settings changes don't seem to apply** — daemon-level settings
  (restore state, auto profile, device path, default FPS,
  quantization) only take effect on daemon start; use the "Restart
  daemon now to apply" button in Settings > Advanced.
- **Autostart toggle does nothing** — it drives `systemctl --user
  enable/disable kbdanim`, which requires `systemd/kbdanim.service` to
  already be installed (`mkdir -p ~/.config/systemd/user && cp
  systemd/kbdanim.service ~/.config/systemd/user/ && systemctl --user
  daemon-reload`).
