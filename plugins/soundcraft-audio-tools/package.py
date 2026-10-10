#!/usr/bin/env python3
"""Package SoundCraft Audio Toolkit ZIP containing channel presets and DSP configs."""
import argparse
import json
import zipfile
from pathlib import Path

PRESETS = {
    "format": "soundcraft.dsp-presets",
    "version": 1,
    "presets": [
        {
            "name": "Broadcast Voice Strip",
            "description": "Standard high-pass filter, subtle compression, and gentle high-shelf boost for speech.",
            "chains": [
                {"dsp": "highpass", "freq": 80.0, "q": 0.707},
                {"dsp": "compressor", "threshold": -18.0, "ratio": 3.0, "attack_ms": 15.0, "release_ms": 120.0},
                {"dsp": "highshelf", "freq": 10000.0, "gain_db": 2.5}
            ]
        },
        {
            "name": "Podcast Master Limiter",
            "description": "Transparent peak limiting with -1.0 dBFS ceiling and gentle dynamic leveling.",
            "chains": [
                {"dsp": "limiter", "ceiling_db": -1.0, "release_ms": 80.0, "lookahead_ms": 5.0}
            ]
        },
        {
            "name": "Acoustic Warmth",
            "description": "Gentle low-mid warmth and soft dynamic smoothing for acoustic recordings.",
            "chains": [
                {"dsp": "bell", "freq": 250.0, "gain_db": 1.5, "q": 1.2},
                {"dsp": "bell", "freq": 3500.0, "gain_db": -1.0, "q": 1.8}
            ]
        }
    ]
}

README_CONTENT = """# SoundCraft Audio Toolkit

Community channel strip presets and DSP configuration for SoundCraft (CLAP, VST3, and built-in mix engine).

## Contents
- `soundcraft-dsp-presets.json`: Channel presets for Broadcast Voice, Podcast Master, and Acoustic Warmth.
- Works with SoundCraft's built-in mix engine and hosted third-party CLAP/VST3/AU plugins.

## Import
Import presets in SoundCraft via Mixer > Channel > Load Preset.
"""

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", default="dist/soundcraft-audio-toolkit.zip")
    args = parser.parse_args()
    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as z:
        z.writestr("README.md", README_CONTENT)
        z.writestr("soundcraft-dsp-presets.json", json.dumps(PRESETS, indent=2) + "\n")
    print(f"Packaged {out}")

if __name__ == "__main__":
    main()
