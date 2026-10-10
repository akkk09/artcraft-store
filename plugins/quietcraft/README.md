# QuietCraft

Automated audio silence detection, noise-floor thresholding, and speech derushing tool for EffectCraft.

## Features
- **Windowed RMS dBFS Detection**: Precise silence and speech detection across customizable audio thresholds.
- **Smart Micro-Pause Bridging**: Configurable minimum silence duration to preserve natural conversational pauses.
- **Noise Burst Suppression**: Minimum speech duration filters out mouth clicks, mic pops, and ambient noise.
- **Attack & Release Padding**: Pre-roll and post-roll margins ensure word onsets and trailing consonants are never clipped.
- **Timeline Editing Modes**:
  - **Ripple Cut (Derush)**: Non-destructively slices layers into speech chunks and shifts them into a continuous tightened edit.
  - **Split at Silence**: Slices layers at silence boundaries for manual review.
  - **Add Markers**: Adds labeled cut markers directly to the layer timeline.
  - **Export Cut List**: Generates standard CMX 3600 EDL, CSV, or JSON cut lists.
- **Interactive ScriptUI Waveform**: Live visualization of audio levels, speech envelope, cut zones, and threshold line.

## Installation
The tool is installed to:
`~/.config/effectcraft/Scripts/ScriptUI Panels/QuietCraft.jsx`

In EffectCraft, open **Window ▸ QuietCraft.jsx** to dock and use the panel.

## Standalone CLI
Run the silence analysis and export commands:
```bash
cargo run --manifest-path plugins/quietcraft/Cargo.toml -- analyze audio.wav --threshold -35
cargo run --manifest-path plugins/quietcraft/Cargo.toml -- edl audio.wav -o cuts.edl
cargo run --manifest-path plugins/quietcraft/Cargo.toml -- presets
```
