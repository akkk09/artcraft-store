# Film Emulation Toolkit

A two-app film-look toolkit:

- **PhotoCraft:** a lightweight ABI v1 Rust/WASM filter with five film-inspired color looks and deterministic monochrome grain.
- **FilmCraft:** a generated ZIP containing five native FilmCraft effect presets (Lumetri grade + built-in Noise effect) and matching 17³ 3D .cube LUTs.

FilmCraft does not currently expose a third-party video-effect plug-in ABI, so its side is an importable preset/LUT pack, not a native executable plug-in.

## PhotoCraft controls

- **Preset:** Classic, Warm, Cool, Faded, or Cinematic.
- **Intensity:** 0–100 for the color grade.
- **Grain:** 0–100 for deterministic monochrome grain.

The PhotoCraft filter supports RGB and Indexed-as-RGB, plus grayscale and Duotone-as-grayscale. Alpha is preserved and fully transparent pixels are left unchanged. This is a lightweight stylized filter, not a calibrated film-stock simulator; it does not model a specific film response curve, halation, or lens behavior.

## FilmCraft pack

The build creates `dist/filmcraft-film-emulation-toolkit.zip` via `package_filmcraft.py`. It contains the native `filmcraft.effect-presets` v1 JSON pack, five matching LUTs, and import instructions.

- Import the JSON pack through FilmCraft's `presets.import` command; each preset combines a Lumetri grade with the built-in Noise effect.
- Alternatively, import a matching .cube file with `lut.import` and select it in Lumetri Color → Creative → Look LUT. LUTs contain color only, while the effect presets also add grain.
- Do not stack a matching LUT on top of its matching grade preset unless you want a stronger look.

The packager checks the preset schema, ZIP CRCs, LUT headers, and expected 17³ row counts.

## Build and test

```sh
rustup target add wasm32-unknown-unknown
cargo test --manifest-path plugins/film-emulation/Cargo.toml
cargo build --manifest-path plugins/film-emulation/Cargo.toml --release --target wasm32-unknown-unknown
python plugins/film-emulation/package_filmcraft.py --output dist/filmcraft-film-emulation-toolkit.zip
```

A successful build does not prove host integration. Install the filter in PhotoCraft and test the FilmCraft presets on representative footage before treating either side as production-ready.
