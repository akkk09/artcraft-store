# Seamless Pattern Generator — PhotoCraft

A PhotoCraft ABI v1 filter that reduces visible seams when a texture is tiled. It blends opposing image edges over a configurable band using mirrored edge samples and a smooth falloff.

## Controls

- **Strength:** 0–100%; 100% makes corresponding opposite-edge pixels match at the boundary.
- **Edge width:** 1–50% of each image dimension; the blend fades smoothly toward the interior.

The filter keeps the original canvas size and preserves alpha. It works on RGB/RGBA and grayscale formats supported by PhotoCraft ABI v1. It is a seam-reduction filter, not a content-aware texture synthesizer: strong corrections can soften or mirror texture near the borders.

## Build and test

```sh
cargo test --manifest-path plugins/seamless-pattern-generator/Cargo.toml
cargo build --manifest-path plugins/seamless-pattern-generator/Cargo.toml --release --target wasm32-unknown-unknown
```
