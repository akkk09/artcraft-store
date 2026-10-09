#!/usr/bin/env bash
set -euo pipefail
TARGET=wasm32-unknown-unknown
mkdir -p dist
for plugin in smudge-blend mixer-blend gradient-map; do
  cargo build --manifest-path "plugins/$plugin/Cargo.toml" --release --target "$TARGET"
done
cp plugins/smudge-blend/target/$TARGET/release/photocraft_plugin_smudge_blend.wasm dist/
cp plugins/mixer-blend/target/$TARGET/release/photocraft_plugin_mixer_blend.wasm dist/
cp plugins/gradient-map/target/$TARGET/release/photocraft_plugin_gradient_map.wasm dist/
cp plugins/chromatic-fringe/chromatic-fringe.wat dist/
printf 'Built plug-ins in dist/\n'
ls -lh dist/*
