# Film Emulation Toolkit

A PhotoCraft ABI v1 filter with five compact film-inspired color looks and deterministic grain.

## Controls

- **Preset:** Classic, Warm, Cool, Faded, or Cinematic.
- **Intensity:** 0–100 for the color grade.
- **Grain:** 0–100 for deterministic monochrome grain.

## Processing and limits

The filter applies a per-pixel color grade and coordinate-based grain. It uses the host image origin coordinates when supplied so grain stays consistent across processing bands. It supports RGB and Indexed-as-RGB, plus grayscale and Duotone-as-grayscale. Alpha is preserved and fully transparent pixels are left unchanged.

This is a lightweight stylized filter, not a calibrated film-stock simulator. It does not model a specific film response curve, halation, or lens behavior. Host installation and visual checks in PhotoCraft are required.

## Build and test

```sh
rustup target add wasm32-unknown-unknown
cargo test --manifest-path plugins/film-emulation/Cargo.toml
cargo build --manifest-path plugins/film-emulation/Cargo.toml --release --target wasm32-unknown-unknown
```
