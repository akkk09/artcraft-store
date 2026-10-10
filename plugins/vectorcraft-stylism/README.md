# VectorCraft Stylism

Vector path offset, live drop shadows, and multi-contour outer/inner glow effects plugin for VectorCraft (ABI v1), inspired by Astute Graphics Stylism.

## Features
- **Path Offset**: Expand or inset closed and open paths by distance $d$ with `round`, `miter`, and `bevel` joins.
- **Drop Shadows**: Generates offset duplicate geometry with configurable displacement $(dx, dy)$ and opacity falloff.
- **Outer / Inner Glow**: Concentric boundary contours with smooth color transitions.

## Build
```bash
cargo build --target wasm32-unknown-unknown --release
```
