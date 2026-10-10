# CaptionCraft

Original subtitle authoring, SRT/WebVTT processor, and caption generator for EffectCraft.

## Features
- **SRT & WebVTT Import/Export**: Robust parser supporting standard millisecond timecodes (`00:01:23,456` and `00:01:23.456`), multi-line text, and Unicode/RTL scripts.
- **Segment Editor & Retiming**: Add, delete, split, merge, and shift subtitle cues.
- **Overlap & Readability Auditor**: Detects timing collisions and provides single-click overlap resolution.
- **Typography & Styling**: Customizable font size, outline/stroke width, fill color, and lower-third title-safe positioning.
- **Interactive Visual Preview**: Embedded vector canvas displaying active subtitle cues over simulated video frames.
- **Timeline Integration**: Automatically generates editable text layers in EffectCraft compositions with exact `inPoint`, `outPoint`, and `TextDocument` styling.

## Installation
The tool is installed to:
`~/.config/effectcraft/Scripts/ScriptUI Panels/CaptionCraft.jsx`

In EffectCraft, open **Window ▸ CaptionCraft.jsx** to dock and use the panel.

## Standalone CLI
Use `captioncraft-cli` for batch subtitle processing:
```bash
cargo run -p captioncraft -- --parse input.srt
cargo run -p captioncraft -- --validate input.srt
cargo run -p captioncraft -- --convert input.srt output.vtt
cargo run -p captioncraft -- --fix-overlaps input.srt fixed.srt
cargo run -p captioncraft -- --shift input.srt shifted.srt 1.5
```
