# PivotCraft

Anchor-point placement, layer alignment, and transform compensation tool for EffectCraft.

## Features
- **9-Point Anchor Grid**: Instant placement to corners (Top-Left, Top-Right, Bottom-Left, Bottom-Right), edges (Top, Bottom, Left, Right), and Center.
- **Custom Location & Percentages**: Set anchor points via arbitrary X/Y/Z percentages (0..100%) and pixel offsets.
- **Visual Position Preservation**: Mathematically verified transform compensation shifts Position values and all existing keyframes so layers never jump or break motion paths.
- **Alpha-Aware Mode**: Detects visible alpha boundaries with a configurable transparency threshold (0..100%).
- **Batch Processing**: Supports multi-layer selections and batch pivot re-alignment in a single undo group.
- **Separated Dimensions**: Automatically detects and compensates separated Position X, Y, and Z properties and their keyframes.
- **2D & 3D Layer Support**: Full 3D orientation, Euler angle (X, Y, Z), and 3D scale compensation for 3D layers.
- **Transform Effect Support**: Option to target native layer transforms or applied Transform effects (`ADBE Transform`).
- **Interactive Visual Preview**: Built-in canvas preview rendering layer boundaries, alpha bounds, current anchor point, and predicted target pivot.
- **Preset Management**: Built-in common alignments and custom user preset saving/reapplication with `app.settings`.

## Installation
The tool is installed to:
`~/.config/effectcraft/Scripts/ScriptUI Panels/PivotCraft.jsx`

In EffectCraft, open **Window ▸ PivotCraft.jsx** to dock and use the panel.

## Standalone CLI
Run the math engine and benchmark:
```bash
cargo run --manifest-path plugins/pivotcraft/Cargo.toml -- --bench
cargo run --manifest-path plugins/pivotcraft/Cargo.toml -- --presets
cargo run --manifest-path plugins/pivotcraft/Cargo.toml -- --solve-2d 50 25 150 150 45
```
