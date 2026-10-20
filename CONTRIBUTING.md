# Contributing to kbdanim

Thanks for considering a contribution! kbdanim is a small, focused
project with a few hard constraints — please read the "Ground rules"
below before sending a PR.

## Ground rules

1. **No new runtime dependencies.** `Cargo.toml`'s `[dependencies]`
   section is empty on purpose (see the comment there). If you think a
   crate is genuinely needed, open an issue to discuss it first.
2. **Animations don't touch hardware.** They implement `Animation` in
   `src/animation.rs` and only ever return a `Frame`. All device I/O
   goes through `KeyboardDevice` (`src/device.rs`).
3. **Single-zone honesty.** This targets one LED with one color and one
   brightness. Please don't add animations whose names/descriptions
   imply movement across physical keys/zones that don't exist on this
   hardware.
4. **No shell subprocess loops.** Sysfs is written via `std::fs`
   directly (see `src/sysfs.rs`), never by shelling out to `tee`/`echo`.
5. **The GUI never touches hardware or runs an animation loop.**
   Everything under `src/gui/` goes through `src/gui/backend.rs`,
   which in turn only ever calls the same library functions the CLI
   uses, or sends the same daemon IPC commands the CLI sends. If a GUI
   feature needs new backend capability, add it to the library first
   (with tests), then wire the GUI to it — don't reach around the
   service layer.
6. Every new animation must be **genuinely** algorithmically distinct
   from existing ones — not a recolor of an existing effect under a new
   name.

## Adding an animation

See "Adding a new animation" in `docs/architecture.md`. In short:
create `src/animations/your_effect.rs`, add `mod your_effect;` plus one
`FACTORIES` line in `src/animations/mod.rs`. Add a row to the table in
`docs/animations.md`. If it's stochastic, seed its RNG from the `seed`
passed to `new()` so `--seed` stays reproducible.

## Before opening a PR

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release

# If you touched anything under src/gui/:
cargo clippy --all-targets --features gui -- -D warnings
cargo build --release --features gui --bin kbdanim-gui
```

All four should be clean. New behavior should come with a unit test
(most modules use `#[cfg(test)] mod tests` at the bottom of the file —
follow the existing style). Hardware-touching code should be tested
against `MockDevice`, not real sysfs.

## Reporting bugs

Please include:
- `kbdanim doctor` output
- The exact command you ran
- If it involves a stochastic animation, the `--seed` you used (or add
  one and reproduce) so the behavior can be replayed exactly.

## License

By contributing, you agree your contribution is licensed under the
project's MIT license (see `LICENSE`).
