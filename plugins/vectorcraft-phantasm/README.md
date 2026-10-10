# VectorCraft Phantasm

Color and tonal grading object filter for VectorCraft (ABI v1), inspired by Astute Graphics Phantasm.

## Features
- **Curves & Levels**: Midtone pivot contrast, gamma, and range adjustments.
- **Exposure**: Precise stops EV scale adjustment.
- **Hue / Saturation / Lightness**: Full perceptual HSL transformations.
- **Color Temperature & Tint**: Kelvin warming/cooling and magenta/green color balance.
- **Invert**: Instant tonal and color inversion.
- **Broad Color Target Support**: Applies to solid fills, stroke colors, linear & radial gradient stops, and embedded raster pixel buffers.

## Build
```bash
cargo build --target wasm32-unknown-unknown --release
```
