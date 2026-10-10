# DepthCraft

Monocular depth estimation, atmospheric fog, and depth map generator plugin for EffectCraft.

## Features
- **Depth Map Modes**:
  - **Grayscale Depth**: Normalized depth map for 3D displacement, camera focus, and depth of field.
  - **Turbo Heatmap**: Perceptually uniform false-color heatmap visualization.
  - **Atmospheric Fog**: Realistic aerial perspective and depth fog with customizable falloff and density.
- **Edge-Preserving Filtering**: Bilateral joint refinement prevents haloing around high-contrast silhouettes.
- **Temporal Consistency**: Motion-adaptive recursive temporal filter suppresses inter-frame flicker in video sequences.
- **WASM Acceleration**: Native WebAssembly plugin ABI v1 compatible with EffectCraft.

## Installation
Copy `depthcraft.wasm` to your EffectCraft plugins folder:
```bash
mkdir -p ~/.config/effectcraft/plugins
cp depthcraft.wasm ~/.config/effectcraft/plugins/
```
In EffectCraft, apply via **Effect ▸ Stylize ▸ DepthCraft**.
