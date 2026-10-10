# LumaSweep

**LumaSweep** is an original, controllable light sweep effect plug-in for EffectCraft (Plug-in API v1). It produces polished highlights on typography, transparent logos, vector silhouettes, and photographic footage with physical bevel edge response, chromatic fringe dispersion, procedural micro-surface texture, and automatic time-based sweep animation.

Inspired by industry-standard light sweep tools like Light Sweep Pro, LumaSweep provides deep creative control over edge normals, lighting models, spectral dispersion, and surface finish.

---

## Key Features

- **Controllable Beam Geometry & Falloff**:
  - Center position with normalized coordinate support.
  - Direction angle in degrees.
  - Beam Width and Softness feathering controls.
  - Five distinct beam profiles: *Smooth Hermite*, *Linear*, *Sharp Peak*, *Gaussian*, and *Asymmetric Leaning Flare*.
  - Sweep intensity boost up to 500%.

- **Bevel Edge Normal Response**:
  - Fast, resolution-aware edge gradient and relief calculation.
  - Adjustable **Bevel Depth** and **Bevel Intensity**.
  - Four curvature profiles: *Curved (Glossy)*, *Chisel (Sharp)*, *Ridge (Stepped)*, and *Rim (Accent)*.
  - Specular glints and rim shadows facing away from the light vector for dimensional depth.

- **Dual Color System (Highlight & Shadow)**:
  - Custom RGBA highlight tinting for specular peaks.
  - Deep shadow / ambient occlusion tinting on trailing edges and bevel borders with adjustable intensity.

- **Optical Dispersion & Glow**:
  - **Chromatic Fringe**: Splitting of the highlight into spectral red and blue divergence along the wavefront.
  - **Glow Halo**: Secondary diffused Gaussian bloom with customizable radius and intensity.

- **Procedural Micro-Surface Texturing**:
  - Deterministic procedural grain generator without heap allocation or external dependencies.
  - Three surface modes:
    - *Brushed Anisotropic*: Elongated metallic striations parallel to the wavefront.
    - *Micro Grain*: 2D isotropic stipple for sandblasted / matte sheen.
    - *Satin Cross*: Interwoven harmonic micro-texture.

- **Time-Based Animation**:
  - Built-in automatic sweep progression using layer time.
  - Adjustable speed (cycles per second), direction phase offset, and loop modes (*One-Way Repeat* vs *Ping-Pong Bounce*).

- **Luminance Relief Mode for Image Sources**:
  - Operates on transparent logos/text (*Alpha mode*), flat photos/artwork (*Luminance mode*), or *Combined hybrid mode*.
  - Flawlessly catches highlights on photographic contours, illustrations, and opaque graphic designs.

- **Original Presets**:
  - **Chrome**: High contrast, crisp blue specular, chisel bevel, mirror rim shadow.
  - **Soft Studio**: Warm key light, smooth cosine falloff, gentle bloom, soft curved bevel.
  - **Prism**: High-intensity iridescent sweep with spectral rainbow chromatic dispersion.
  - **Brushed Metal**: Directional anisotropic striations, steel highlight, medium bevel.
  - **Gold Lustre**: Rich 24k gold specular ([1.0, 0.88, 0.45]) with deep bronze shadow.
  - **Laser Beam**: Narrow high-intensity neon cyan flare with heavy glow and cutout light.

- **Resolution-Aware & Transparent**:
  - Seamlessly scales beam width, bevel radius, and texture frequency with preview scale factor.
  - Un-premultiplies for lighting calculations and cleanly re-premultiplies on output.
  - Supports *Add*, *Composite / Blend*, *Cutout (Light Only)*, and *Screen* reception modes.

---

## Building from Source

Requirements: Rust stable with target `wasm32-unknown-unknown`.

```bash
# Run unit and integration tests
cargo test

# Build WebAssembly plug-in
cargo build --release --target wasm32-unknown-unknown

# Run CLI benchmarks
cargo run --bin lumasweep-cli -- --bench
```

---

## Installation in EffectCraft

Copy `lumasweep.wasm` to the EffectCraft plug-ins directory:

```bash
mkdir -p ~/.config/effectcraft/plugins/
cp lumasweep.wasm ~/.config/effectcraft/plugins/
```

In EffectCraft:
- Apply via **Effect ▸ Generate ▸ LumaSweep**.
- Or via EffectCraft CLI: `effectcraft-cli exec effect.apply '{"layer": 1, "effect": "LumaSweep"}'`.
