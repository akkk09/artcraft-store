#!/usr/bin/env bash
set -euo pipefail
TARGET=wasm32-unknown-unknown
mkdir -p dist
for plugin in smudge-blend mixer-blend gradient-map color-toolkit detail-recovery film-emulation mosaic-pixelate vignette white-balance lut-studio geometry-fixer seamless-pattern-generator depthcraft lumasweep vectorcraft-recolor; do
  cargo build --manifest-path "plugins/$plugin/Cargo.toml" --release --target "$TARGET"
done
cp plugins/smudge-blend/target/$TARGET/release/photocraft_plugin_smudge_blend.wasm dist/
cp plugins/mixer-blend/target/$TARGET/release/photocraft_plugin_mixer_blend.wasm dist/
cp plugins/gradient-map/target/$TARGET/release/photocraft_plugin_gradient_map.wasm dist/
cp plugins/color-toolkit/target/$TARGET/release/photocraft_plugin_color_toolkit.wasm dist/
cp plugins/detail-recovery/target/$TARGET/release/photocraft_plugin_detail_recovery.wasm dist/
cp plugins/film-emulation/target/$TARGET/release/photocraft_plugin_film_emulation.wasm dist/
cp plugins/mosaic-pixelate/target/$TARGET/release/photocraft_plugin_mosaic_pixelate.wasm dist/
cp plugins/vignette/target/$TARGET/release/photocraft_plugin_vignette.wasm dist/
cp plugins/white-balance/target/$TARGET/release/photocraft_plugin_white_balance.wasm dist/
cp plugins/lut-studio/target/$TARGET/release/photocraft_plugin_lut_studio.wasm dist/
cp plugins/geometry-fixer/target/$TARGET/release/photocraft_plugin_geometry_fixer.wasm dist/
cp plugins/seamless-pattern-generator/target/$TARGET/release/photocraft_plugin_seamless_pattern_generator.wasm dist/
cp plugins/depthcraft/target/$TARGET/release/depthcraft.wasm dist/ 2>/dev/null || cp plugins/depthcraft/depthcraft.wasm dist/
cp plugins/lumasweep/target/$TARGET/release/lumasweep.wasm dist/ 2>/dev/null || cp plugins/lumasweep/lumasweep.wasm dist/
cp plugins/vectorcraft-recolor/target/$TARGET/release/vectorcraft_plugin_recolor.wasm dist/
python plugins/film-emulation/package_filmcraft.py --output dist/filmcraft-film-emulation-toolkit.zip
python plugins/creator-graphics-pack/package.py --output dist/creator-graphics-pack.zip
python plugins/soundcraft-audio-tools/package.py --output dist/soundcraft-audio-toolkit.zip
cp plugins/chromatic-fringe/chromatic-fringe.wat dist/
cp plugins/pdfcraft-form-tools/pdfcraft-invoice-calculator.js dist/
cp plugins/easecraft/EaseCraft.jsx dist/
cp plugins/captioncraft/CaptionCraft.jsx dist/
cp plugins/quietcraft/QuietCraft.jsx dist/
printf 'Built plug-ins in dist/\n'
ls -lh dist/*
