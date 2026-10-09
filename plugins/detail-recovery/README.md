# Detail Recovery

A PhotoCraft ABI v1 filter with lightweight sharpening and noise reduction.

## Controls

- **Sharpen:** 0–100. Adds local high-frequency detail using a 3×3 neighborhood.
- **Denoise:** 0–100. Moves each opaque pixel toward the local 3×3 mean. This is a simple spatial smoother, not an AI or frequency-domain denoiser.

## Processing and limits

The filter uses a one-pixel overlap and a 3×3 box neighborhood. It supports RGB and Indexed-as-RGB, plus grayscale and Duotone-as-grayscale. Alpha is preserved; fully transparent pixels are unchanged and transparent neighbors are excluded from the local mean to reduce edge halos.

The operation is intentionally small and fast. Strong denoise settings can soften fine texture, and strong sharpening can create halos or values outside the 0–1 range in the intermediate buffer. PhotoCraft clamps integer-depth output as documented by ABI v1. Try modest settings first.

## Build and test

```sh
rustup target add wasm32-unknown-unknown
cargo test --manifest-path plugins/detail-recovery/Cargo.toml
cargo build --manifest-path plugins/detail-recovery/Cargo.toml --release --target wasm32-unknown-unknown
```

A successful build does not prove host integration. Install the filter in PhotoCraft and test on photos with both fine texture and smooth areas before treating it as production-ready.
