# VectorCraft MirrorMe

Real-time symmetry, axis reflection, and kaleidoscopic vector replication plugin for VectorCraft (ABI v1), inspired by Astute Graphics MirrorMe.

## Features
- **Arbitrary Axis Reflection**: Mirror across horizontal, vertical, or any angle $\theta \in [-180^\circ, 180^\circ]$.
- **Configurable Pivot Point**: Set custom center coordinates $(p_x, p_y)$.
- **Multi-Axis & Kaleidoscopic Symmetry**: 1 to 12 symmetry lines generating rotational-reflective radial patterns.
- **Full Geometry Transformation**: Accurately reflects SVG path data strings (`M`, `L`, `C`, `Z`), point coordinate lists, and child hierarchies while preserving styles.

## Build
```bash
cargo build --target wasm32-unknown-unknown --release
```
