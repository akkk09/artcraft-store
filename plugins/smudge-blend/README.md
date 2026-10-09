# Smudge Blend

PhotoCraft ABI v1 filter plug-in. It blends each pixel toward a directional sample behind it.

## Controls
- **Strength**: effect amount.
- **Radius**: sampling distance.
- **Direction**: right, left, up, down, down-right, or down-left.

This is a whole-layer filter approximation, not a drag-to-smudge brush. ABI v1 does not provide pointer events or per-stroke state.

Build from the repository root with `./build-all.sh`. Output: `dist/photocraft_plugin_smudge_blend.wasm`.
