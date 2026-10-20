# kbdanim architecture

## Design goals

- **Single-zone honesty.** The target hardware is one LED with one
  color and one brightness — not an addressable per-key matrix. Every
  animation is built from exactly two knobs (global color, global
  brightness) modulated over time; none of them pretend to move light
  "across" a keyboard that has no physical zones to move across (see
  `docs/animations.md`).
- **Zero runtime dependencies.** Config parsing, CLI parsing, the PRNG,
  IPC, and signal handling are all implemented directly on `std` plus a
  handful of hand-written libc bindings (`src/ffi.rs`). This keeps the
  binary small, the build reproducible, and removes an entire class of
  supply-chain risk for a program that runs continuously with device
  access.
- **Hardware and animation logic never touch.** Animations implement
  `Animation::update(time, delta, context) -> Frame` and know nothing
  about sysfs. The engine (`src/engine.rs`) is the only thing that talks
  to `KeyboardDevice`. This is what makes `MockDevice` (used by the
  whole test suite) possible.

## Module map

kbdanim is a library crate (`src/lib.rs`) plus two binaries that
depend on it: `src/main.rs` (the `kbdanim` CLI/daemon, always built)
and `src/gui/main.rs` (the optional `kbdanim-gui`, built only with
`--features gui`). Both binaries call the exact same backend code
below — there is no second implementation of anything hardware- or
protocol-related for the GUI. See `docs/gui.md` for the GUI's own
internal structure (`src/gui/{app,backend,theme}.rs` and
`src/gui/views/`).

```
main.rs         CLI entry point and command dispatch
cli.rs          Flag parsing (no external arg-parsing crate)
error.rs        Crate-wide error type

color.rs        RGB/HSV/HSL, named colors, parsing, interpolation
easing.rs       Easing curves
rng.rs          SplitMix64 PRNG
timing.rs       Drift-corrected frame pacing (monotonic Instant)
brightness.rs   animation-brightness * global-brightness = final value

animation.rs    Animation trait, Frame, AnimationParams/Context
animations/     One file per animation + a shared registry (mod.rs)
engine.rs       Ties timing + animation + device together each frame

device.rs       KeyboardDevice trait; SysfsDevice + MockDevice
sysfs.rs        Raw sysfs file I/O (no shell subprocess, ever)

toml_lite.rs    Minimal TOML-subset parser (see its own doc comment)
config.rs       Config struct + loading + per-power-profile settings
profile.rs      Profile lookup helpers
preset.rs       Built-in + user preset loading
palette.rs      Built-in + user palette loading
paths.rs        XDG-compliant path helpers
state.rs        Daemon runtime state (shared, mutex-guarded) + persistence
power.rs        AC/battery detection via /sys/class/power_supply
logging.rs      Leveled stderr logger (--verbose/--quiet)

daemon.rs       The foreground daemon loop
ipc.rs          Unix-socket line protocol between the CLI and the daemon
signal.rs       SIGINT/SIGTERM -> clean shutdown
ffi.rs          Hand-written getuid()/signal() bindings
```

## Data flow for one frame

```
CLI / daemon loop
      │
      ▼
Engine::step(animation, params, device)
      │
      ├─ advance sim_time by real_delta * params.speed
      ├─ animation.update(sim_time, delta, ctx) -> Frame{color, brightness}
      ├─ quantize_color(frame.color, config.quantization)
      ├─ brightness::combine(frame.brightness, global_brightness)
      └─ device.write_color(...) / device.write_brightness(...)
             │
             ▼
      SysfsDevice (write-diff cached) -> /sys/class/leds/.../{brightness,multi_intensity}
```

`Engine::pace()` is called once per loop iteration, after `step()`, and
sleeps using a drift-corrected deadline (`timing::FrameTimer`) rather
than a naive `sleep(1/fps)` — see that module's doc comment for why.

## Daemon vs. foreground `run`

`kbdanim run` and `kbdanim daemon` share the exact same `Engine` and
animation code. The difference is only in how they're controlled:

- `run` owns one `AnimationParams` for its whole lifetime, set once from
  CLI flags. There's no IPC; it's meant for foreground testing/one-offs
  and exits cleanly on Ctrl+C.
- `daemon` holds its parameters in a `Mutex`-guarded `RuntimeState`,
  mutated by an IPC thread-of-control embedded in the same loop (see
  "Why foreground instead of a classic daemon?" at the top of
  `daemon.rs`), and additionally handles power-profile auto-switching,
  random-mode, and periodic state persistence.

## Adding a new animation

1. Create `src/animations/my_effect.rs` implementing the `Animation`
   trait (see any existing file, e.g. `oscillation.rs`, for the
   shortest template).
2. Add `mod my_effect;` to `src/animations/mod.rs`.
3. Add one line to the `FACTORIES` array in the same file:
   `("my-effect", |seed| Box::new(my_effect::MyEffect::new(seed))),`

Nothing else needs to change — `list`, `search`, `doctor`, shell
completions (which are static — update them too if you want tab
completion for the new name), and the daemon's `random-mode` all read
from the same registry.
