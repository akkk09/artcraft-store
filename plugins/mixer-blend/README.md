# Mixer Blend

PhotoCraft ABI v1 filter plug-in that approximates wet-paint mixing by blending each pixel with a circular local colour average.

## Controls
- **Strength**: effect amount.
- **Radius**: neighborhood size.
- **Wetness**: how much of the local mix influences the result.
- **Pickup**: how strongly nearby colours contribute.

This is a whole-layer filter approximation, not an interactive brush that carries paint across a mouse stroke. ABI v1 does not provide pointer events or persistent per-stroke state.

Build from the repository root with `./build-all.sh`. Output: `dist/photocraft_plugin_mixer_blend.wasm`.
