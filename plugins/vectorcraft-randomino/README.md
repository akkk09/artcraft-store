# VectorCraft Randomino

Generative variation and randomization plugin for VectorCraft (ABI v1), inspired by Astute Graphics Randomino.

## Features
- **Deterministic Seed**: Reproducible variations with 64-bit XorShift PRNG.
- **Transform Randomization**:
  - Position jitter ($\Delta x, \Delta y$).
  - Rotation around each object's geometric centroid ($\theta \in [\theta_{\min}, \theta_{\max}]$).
  - Scale factor ($s \in [s_{\min}, s_{\max}]$, uniform or independent axes).
- **Style Variation**:
  - Perceptual HSL color variance ($\Delta H, \Delta S, \Delta L$).
  - Opacity jitter ($\alpha \in [\alpha_{\min}, \alpha_{\max}]$).
- **Z-Stacking Shuffling**: Fisher-Yates layer shuffling for complex layered compositions.

## Build
```bash
cargo build --target wasm32-unknown-unknown --release
```
