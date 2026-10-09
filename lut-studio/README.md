# LUT Studio

A static browser-based 3D LUT workspace. It runs locally in the browser and does not upload media or LUT files.

## Features
- Import standard 3D `.cube` LUTs (sizes 2–33; supports `TITLE`, `DOMAIN_MIN`, `DOMAIN_MAX`, and comments; rejects 1D LUTs and malformed tables).
- Five original sample looks matching the Film Emulation Toolkit family: Classic, Warm, Cool, Faded, and Cinematic.
- Trilinear interpolation, 0–100% intensity, before/after preview, and image or video-frame preview.
- Save/delete custom LUTs in browser local storage.
- Export a `.cube` LUT or the processed image/current video frame as PNG.

## Use
Open `/lut-studio/`, select an image or video, then choose a sample look or import a `.cube` file. Everything runs client-side. Exporting a video creates a PNG of the current frame, not a rendered video.

## Compatibility
The PhotoCraft ABI v1 adapter ships with five bundled looks. Its host API does not expose arbitrary file access, so custom LUT import is provided by this browser tool. FilmCraft support is a separate follow-up and is not claimed until verified.
