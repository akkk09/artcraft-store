# LUT Studio — PhotoCraft adapter

PhotoCraft ABI v1 filter adapter with five original 17³ 3D LUTs (Classic, Warm, Cool, Faded, Cinematic) and a 0–100% intensity control. Build-time validation parses the accompanying .cube files; the shared no-std parser and trilinear sampler have independent tests.

The PhotoCraft ABI exposes image pixels and scalar/choice parameters, but no arbitrary filesystem/file-upload API. Therefore this adapter selects bundled looks. Custom .cube import, previewing, and exporting are provided by the companion browser tool at `lut-studio/`. FilmCraft support is a separate follow-up and is not claimed here.

## Build and test

```sh
cargo test --manifest-path plugins/lut-studio/Cargo.toml
cargo build --manifest-path plugins/lut-studio/Cargo.toml --release --target wasm32-unknown-unknown
```
