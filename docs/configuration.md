# Configuration reference

kbdanim follows the XDG Base Directory spec:

| Purpose | Path | Helper |
|---|---|---|
| Config file | `$XDG_CONFIG_HOME/kbdanim/config.toml` (usually `~/.config/kbdanim/config.toml`) | `kbdanim config path` |
| Presets | `$XDG_CONFIG_HOME/kbdanim/presets/*.toml` | `kbdanim preset list` |
| Palettes | `$XDG_CONFIG_HOME/kbdanim/palettes/*.toml` | `--palette <name>` |
| Playlists | `$XDG_CONFIG_HOME/kbdanim/playlists/*.toml` | `kbdanim playlist run <name>` |
| Persisted state | `$XDG_STATE_HOME/kbdanim/state.toml` (usually `~/.local/state/kbdanim/state.toml`) | restored automatically by the daemon |
| IPC socket | `$XDG_RUNTIME_DIR/kbdanim/kbdanim.sock` | used internally by the CLI |

Run `kbdanim config edit` to create `config.toml` from the shipped
template (see `config/default.toml` in this repo) and open it in
`$EDITOR` (falls back to `nano`).

## `config.toml` format

kbdanim does **not** depend on the `toml` crate; it implements a
deliberately minimal TOML subset (see the doc comment at the top of
`src/toml_lite.rs`) that covers everything the config/preset/palette/
playlist/state files need: comments, `[section]` and `[section.sub]`
headers, `[[array.of.tables]]`, and string/int/float/bool/string-array
values. If a file fails to parse, kbdanim logs a warning and falls back
to built-in defaults rather than crashing (spec: config parse failures
must never be fatal).

```toml
device = "/sys/class/leds/rgb:kbd_backlight"

[animation]
mode = "rainbow"
speed = 1.0
fps = 30
brightness = 200
easing = "smoothstep"
color = "#00AAFF"
color2 = "#FF0080"

[daemon]
enabled = false
restore_state = true

[power]
auto_profile = true

[transition]
enabled = true
duration = 1.5

[engine]
color_quantization = 1

[random]
interval = 30.0

[profiles.ac]
mode = "rainbow"
brightness = 255
speed = 1.0
fps = 30

[profiles.battery]
mode = "breathing"
brightness = 80
speed = 0.8
fps = 15
```

Every key is optional; anything omitted uses kbdanim's built-in
default (shown in the table above / in `Config::default()`).

## Presets vs. profiles

- A **preset** (`kbdanim preset run <name>`) is a single, complete
  animation configuration (mode + brightness + speed + fps + optional
  color/easing) that you apply on demand.
- A **profile** (`[profiles.*]` in config.toml) describes the device's
  overall behavior in a given context (plugged in, on battery, gaming,
  night) and can be switched automatically by the daemon based on power
  state (`[power].auto_profile = true`), or manually via
  `kbdanim profile switch <name>` / `kbdanim power-profile ac|battery`.

## Preset file format

```toml
name = "my-preset"
mode = "fire"
brightness = 220
speed = 1.1
fps = 30
color = "#ff6600"     # optional
color2 = "#ffcc00"    # optional
easing = "ease-in-out" # optional
```

## Palette file format

```toml
name = "my-palette"
colors = ["#ff0000", "#ff8800", "#ffff00", "#ffffff"]
```

At least two colors are required. Use with `--palette my-palette` on
the `color-cycle` or `rgb-cycle` animations.

## Playlist file format

```toml
[[animations]]
mode = "sunset"
duration = 45

[[animations]]
mode = "aurora"
duration = 45
```

Run with `kbdanim playlist run <name>` (reads
`$XDG_CONFIG_HOME/kbdanim/playlists/<name>.toml`). The playlist repeats
until stopped with Ctrl+C.
