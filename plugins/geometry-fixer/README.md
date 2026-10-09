# Geometry Fixer — PhotoCraft

A lightweight PhotoCraft ABI v1 geometry filter for correcting slightly tilted or perspective-skewed images without dependencies.

## Controls
- **Rotation:** -15° to +15° for straightening.
- **Horizontal perspective:** -100 to +100 to adjust horizontal convergence.
- **Vertical perspective:** -100 to +100 to adjust vertical convergence.
- **Zoom:** 50% to 200% to control framing after correction.

The filter uses inverse mapping and bilinear sampling. It keeps the original canvas dimensions; pixels mapped outside the source become transparent when alpha is available, otherwise black. It preserves alpha through interpolation and supports RGB/RGBA and grayscale formats supported by PhotoCraft ABI v1.

## Build and test

```sh
cargo test --manifest-path plugins/geometry-fixer/Cargo.toml
cargo build --manifest-path plugins/geometry-fixer/Cargo.toml --release --target wasm32-unknown-unknown
```

This is a fixed-canvas perspective/rotation correction filter, not a content-aware cropper or a lens-profile database.
