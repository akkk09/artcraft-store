# EaseCraft

Interactive visual easing curve editor, mathematical transition engine, and preset manager for EffectCraft.

## Features
- **Visual Cubic Bézier Graph View**: 0..1 time, -0.35..1.35 value with reference grid and tangent handles.
- **Dynamic Mathematical Presets**:
  - Linear, Ease In, Ease Out
  - Ease In-Out (Soft, Regular, Strong, Extreme)
  - Back In (Anticipation), Back Out (Overshoot), Back In-Out
  - Elastic Snap, Bounce Settle, Smooth S-Curve
- **Animated Preview Marker**: Live motion tracker and preview ball demonstrating curve physics.
- **Curve Operations**: Mirror, Reverse, From Keys (Capture), Copy, Paste.
- **Multidimensional & Spatial Support**: 1D, 2D, 3D, and curved 3D motion paths with spatial tangent integration.
- **Open Format**: Standard JSON preset interchange schema.

## Installation
The tool is installed to:
`~/.config/effectcraft/Scripts/ScriptUI Panels/EaseCraft.jsx`

In EffectCraft, open **Window ▸ EaseCraft.jsx** to dock and use the panel.

## Standalone CLI
Run the math engine and benchmark:
```bash
cargo run --release -p easecraft -- --bench
cargo run --release -p easecraft -- --presets
```
