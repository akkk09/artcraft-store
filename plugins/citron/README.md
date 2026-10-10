# Citron: Professional Multi-Channel Graph Editor for EffectCraft

**Citron** is a 3D animation software style (Maya, Blender, Cinema 4D) curve workspace and graph editor extension for EffectCraft. Designed for motion designers, character animators, and VFX artists, it replaces single-property easing tools with a multi-channel curve-shaping environment.

## Key Features

- **Multi-Channel Editing:** Inspect, compare, and shape multiple animated properties simultaneously (e.g. Position X, Y, Z, Rotation, Scale, Opacity, and effect controls) in a unified color-coded viewport.
- **Multiple View Modes:**
  - **Normalized (0% – 100%):** Compares disparate units (e.g. degrees vs pixels vs percentage) on a normalized vertical scale.
  - **Absolute Values:** Standard physical coordinate view.
  - **Stacked Lanes:** Automatically partitions curves into non-overlapping horizontal bands.
  - **Speed Graph:** Displays first-derivative velocity curves over time.
- **Free-Form Deformation (FFD) Lattice Cage:** Bounding cage tool over selected keyframes enabling bulk retiming, amplitude scaling, time shifting, and progressive non-linear skew.
- **De Casteljau Subdivision:** Insert keyframes anywhere along an active curve without altering trajectory or easing dynamics.
- **Buffer & Ghost Curves:** Captures an instant snapshot of curves before manipulation, rendering ghosted reference curves to evaluate before/after motion timing.
- **Unified vs. Broken Tangents:** Toggle between collinear smooth tangents and sharp broken angles.
- **Preset Tangent Accelerations:** One-click presets for Bezier Smooth, Linear, Stepped Hold, Auto-Clamp (extrema flattening), Easy Ease (33%), Dynamic Punch (75%), and Overshoot.
- **Tangent Clipboard:** Copy and paste tangent weights and velocities across any keyframes or channels.
- **Full Undo Stack Integration:** Every transformation is grouped into native EffectCraft undo history.

## Installation

Place `Citron.jsx` into your EffectCraft ScriptUI Panels directory:

```bash
cp Citron.jsx ~/.config/effectcraft/Scripts/ScriptUI\ Panels/
```

Launch EffectCraft and open **Window ▸ Citron.jsx**. Dock the panel adjacent to the Timeline or Comp viewer.

## CLI Utility

The companion `citron-cli` binary provides standalone benchmark and curve-solving utilities:

```bash
cargo run --release --bin citron-cli -- bench
```
