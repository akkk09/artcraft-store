# Gradient Map & Duotone

PhotoCraft ABI v1 filter that maps luminance to one of five built-in two-colour gradients.

## Controls

- **Preset:** Black to white, Warm cream, Blue to orange, Purple to pink, or Teal to yellow.
- **Intensity:** 0–100; 0 leaves the input unchanged and 100 applies the full mapping.

## Behaviour and limits

- Uses the PhotoCraft ABI v1 interleaved f32 pixel buffer and manifest-driven parameters.
- Supports RGB and Indexed-as-RGB documents, plus grayscale and Duotone-as-grayscale documents.
- Preserves the alpha channel and skips fully transparent pixels.
- Uses Rec. 709 luminance coefficients on the supplied samples and deterministic linear interpolation.
- Returns an error for unsupported colour modes or channel layouts rather than guessing how to convert them.
- ABI v1 filters preserve input dimensions; geometric operations are out of scope.

## Build

From the repository root, install the wasm32-unknown-unknown Rust target and run:

cargo build --manifest-path plugins/gradient-map/Cargo.toml --release --target wasm32-unknown-unknown

The release artifact is written under plugins/gradient-map/target/wasm32-unknown-unknown/release/. The store build script copies it to dist/.

## Tests

The crate contains unit tests for interpolation endpoints and bounds, luminance endpoints, parameter clamping, and preset selection. Run native unit tests where supported, then compile the actual WebAssembly target. Host integration must be checked in PhotoCraft; a successful build alone does not prove the filter loads or renders correctly.
