# Animation catalog

kbdanim ships 34 animations. All of them operate on a single LED zone:
"movement" is always expressed as color and/or brightness changing over
time, never as light moving between physical positions that don't
exist on this hardware (see `docs/architecture.md`).

Every animation reads `--color`/`--color2`/`--easing`/`--period`/
`--saturation` where relevant; a few also read `--randomness`,
`--intensity`, `--temperature`, `--palette`, or `--seed`. Defaults for
`--color`/`--color2` are chosen per-animation (see `default_colors_for`
in `src/animation.rs`) so e.g. `kbdanim run --mode police` is red/blue
out of the box without having to specify colors manually.

| Name | Category | What it does |
|---|---|---|
| `static` | Basic | Solid, unchanging color. |
| `breathing` | Basic | Smooth sine-wave brightness fade. |
| `pulse` | Basic | Sharp attack, slower decay, repeating. |
| `heartbeat` | Basic | Two quick beats then a pause. |
| `strobe` | Basic | Hard, un-eased on/off flashing. |
| `rainbow` | Color | Continuous linear hue rotation. |
| `smooth-rainbow` | Color | Eases between 60-degree hue keyframes. |
| `color-cycle` | Color | Crossfades through a palette (`--palette` or `--color`/`--color2`). |
| `random` | Color | Smoothly interpolated random vivid colors. |
| `random-pastel` | Color | Smoothly interpolated random pastel colors. |
| `breathing-color` | Color | Breathing brightness + slow continuous hue rotation, combined. |
| `warm` | Color | Amber-gold hue wobble + independent brightness wobble. |
| `cool` | Color | Cyan-blue hue wobble + independent brightness wobble. |
| `night` | Color | Very low brightness ceiling, slow blue-violet glow. |
| `fire` | Nature | Warm hue random-walk + fast stochastic brightness flicker. |
| `candle` | Nature | Narrower amber band, slower/gentler flicker than fire. |
| `ocean` | Nature | Slow blue-teal swell from two layered sine waves. |
| `forest` | Nature | Slow greens with a dappled-light brightness wave. |
| `aurora` | Nature | Slow multi-color hue drift across green/teal/purple. |
| `sunset` | Nature | Crossfades a fixed gold -> pink -> purple -> deep-blue gradient. |
| `lava` | Nature | Slower/darker than fire, with occasional bright glow spikes. |
| `ice` | Nature | Pale, high-brightness, low-saturation shimmer. |
| `meteor` | Effect | Periodic bright flash exponentially decaying into a trail color. |
| `lightning` | Effect | Near-black baseline, short irregular spikes, occasional double-strike. |
| `sparkle` | Effect | Frequent brief twinkles on a dim base. |
| `oscillation` | Effect | Constant-brightness ping-pong between `color`/`color2`. |
| `wave` | Effect | Two sine harmonics drive color mix and brightness together. |
| `plasma` | Effect | Classic sum-of-sines organic hue field. |
| `cyberpunk` | Style | Magenta/cyan crossfade with brief random glitch dips. |
| `matrix` | Style | Fast deterministic green flicker (layered incommensurate sines). |
| `toxic` | Style | Acid green-yellow pulse with a hue wobble. |
| `party` | Style | Fast, hard-cut random vivid color jumps. |
| `police` | Style | Alternating red/blue double-flash bursts. |
| `rgb-cycle` | Style | Hard (non-crossfaded) switching between red/green/blue or a palette. |

## Notable parameter usage

- **`--seed`**: `random`, `random-pastel`, `fire`, `candle`, `lava`,
  `meteor`, `lightning`, `sparkle`, `party`, and `cyberpunk` all use an
  internal PRNG seeded from `--seed` (default: derived from `0`, i.e.
  deterministic unless you pass a seed explicitly at the CLI's
  discretion). Pass the same seed to reproduce a run exactly, which is
  especially useful when filing a bug report.
- **`--palette`**: used by `color-cycle` and `rgb-cycle`; falls back to
  `[--color, --color2]` (or `[red, green, blue]` for `rgb-cycle`) when
  omitted.
- **`--intensity`**: scales flicker/random-walk speed for `fire`,
  `candle`, `lava`, `lightning`, `sparkle`, `party`, and hue-motion speed
  for `plasma`/`matrix`.

## Adding your own

See "Adding a new animation" in `docs/architecture.md`.
