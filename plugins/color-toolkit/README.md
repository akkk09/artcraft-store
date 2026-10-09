# Color Toolkit

A PhotoCraft ABI v1 filter for exposure, contrast, and saturation adjustments.

## Controls

- **Exposure:** -3.00 to +3.00 stops, represented as an integer from -300 to 300 in hundredths of a stop.
- **Contrast:** -100 to +100. Zero keeps the input unchanged; -100 maps color channels to mid-gray before saturation.
- **Saturation:** 0 to 200 percent. 100 is unchanged, 0 removes chroma, and 200 doubles chroma relative to luminance.

## Processing

Operations run in this order: exposure, contrast around 0.5, then saturation around Rec. 709 luminance. RGB and Indexed-as-RGB inputs use all three controls. Grayscale and Duotone-as-grayscale inputs use exposure and contrast; saturation has no effect. The filter preserves alpha and leaves fully transparent pixels unchanged.

The filter supports PhotoCraft ABI v1 mode values 1 (Grayscale), 2 (Indexed stored as RGB), 3 (RGB), and 8 (Duotone stored as grayscale). Other color modes return an error rather than being converted implicitly. Samples stay in the host's normalized f32 representation; 32-bit documents may contain values above 1.

## Build

From the repository root:

```sh
rustup target add wasm32-unknown-unknown
cargo build --manifest-path plugins/color-toolkit/Cargo.toml --release --target wasm32-unknown-unknown
```

## Tests

Run native algorithm tests with:

```sh
cargo test --manifest-path plugins/color-toolkit/Cargo.toml
```

A successful build does not prove host integration. Install and run the filter inside PhotoCraft before marking it ready for users.
