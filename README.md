# PhotoCraft Plugin Store

A community catalog of plug-ins for [PhotoCraft](https://github.com/storytold/photocraft).

## Available plug-ins

| Plug-in | Type | What it does |
|---|---|---|
| **Smudge Blend** | Filter (ABI v1) | Smears pixels in a chosen direction with adjustable strength and radius. |
| **Mixer Blend** | Filter (ABI v1) | Mixes nearby colours with adjustable pickup, radius, and wetness. |

### Important compatibility note

PhotoCraft's current public plug-in ABI v1 supports **whole-buffer filters only**. It does not expose pointer/mouse-drag events, brush cursors, or persistent per-stroke state. Therefore these entries are smudge-/mixer-inspired filters, **not interactive brush tools** like Photoshop's brushes. A true drag-to-paint brush requires changes to PhotoCraft's core tool system.

## Install

1. Download a `.wasm` asset from a plug-in's GitHub Release.
2. In PhotoCraft, choose **Filter → Plug-ins → Install Plug-in…** and select the `.wasm` file.
3. The filter appears under **Filter → Plug-ins**.

The repository's GitHub Actions workflow builds both plug-ins for `wasm32-unknown-unknown` and publishes the `.wasm` files when a version tag (for example, `v0.1.0`) is pushed.

## Catalog

See [`catalog.json`](catalog.json) for machine-readable plug-in metadata.

## Build locally

Requirements: Rust stable and `rustup`.

```sh
rustup target add wasm32-unknown-unknown
./build-all.sh
```

Built modules are written to each plug-in's `target/wasm32-unknown-unknown/release/` directory.

## License

MIT. Community project; not affiliated with the PhotoCraft maintainers.
