# ArtCraft Store

A shared, searchable community catalog for extensions across the ArtCraft creative apps:

- **PhotoCraft** — plug-ins, filters, and tools
- **FilmCraft** — plug-ins, effects, and presets
- **EffectCraft** — scripts and plug-ins
- **VectorCraft** — plug-ins, tools, and presets

The static storefront in [`index.html`](index.html) reads [`catalog.json`](catalog.json) at runtime. Filter by app or search across names, descriptions, tags, and compatibility notes.

## Store website

1. Open **Settings → Pages** in this repository.
2. Under **Build and deployment**, select **GitHub Actions** as the source.
3. Push to `main` or manually run **Deploy plugin store website** from Actions.
4. Open the deployment URL shown in the workflow.

## Adding an item

Add an entry to `catalog.json` under `plugins`. Supported fields include `id`, `app`, `name`, `version`, `author`, `description`, `kind`, `tags`, `compatibility`, `sourceUrl`, `downloadUrl`, `artifact`, and `releaseAsset`. The `app` value should be one of `photocraft`, `filmcraft`, `effectcraft`, or `vectorcraft`. Use `sourceUrl` and `downloadUrl` for extensions hosted outside this repository. The storefront supports app-specific formats; it does not assume every item is a WASM file.

Do not add a listing until its source, license, download, and target-app compatibility have been checked.

## Current PhotoCraft plug-ins

| Plug-in | Type | What it does |
|---|---|---|
| **Smudge Blend** | Filter (ABI v1) | Smears pixels in a chosen direction with adjustable strength and radius. |
| **Mixer Blend** | Filter (ABI v1) | Mixes nearby colours with adjustable pickup, radius, and wetness. |

The current build workflow only compiles the PhotoCraft Rust/WASM plug-ins. FilmCraft, EffectCraft, and VectorCraft listings can use their own source repositories and download URLs; their native build/install workflows should be integrated separately once the relevant APIs and formats are confirmed. No unverified listings are fabricated for those apps.

## Build PhotoCraft plug-ins locally

Requirements: Rust stable and `rustup`.

```sh
rustup target add wasm32-unknown-unknown
bash build-all.sh
```

Built modules are copied to `dist/`.

## License

MIT. Community project; not affiliated with the ArtCraft app maintainers.
