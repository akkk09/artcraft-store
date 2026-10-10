# VectorCraft ColliderScribe (Super Marquee)

Advanced marquee selection and geometric query tool for VectorCraft (ABI v1), inspired by Astute Graphics ColliderScribe.

## Features
- **Flexible Marquee Geometry**: Rectangular and elliptical/circular marquee queries.
- **Enclosure vs Intersecting**: Select strictly enclosed objects or any object touching the marquee boundary.
- **Advanced Selection Patterns**:
  - `all`: All matching objects.
  - `alternate`: Stride $N$ (select every 2nd or $N$-th object).
  - `random`: Probabilistic selection of a random subset of objects.
- **Actions**:
  - `mark_selected`: Sets `"selected": true/false` attribute in the document AST.
  - `isolate`: Keeps only selected objects and drops the rest.
  - `exclude`: Removes selected objects from the composition.

## Build
```bash
cargo build --target wasm32-unknown-unknown --release
```
