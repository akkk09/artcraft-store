# PhotoCraft Plugin Store

A community catalog of plug-ins for [PhotoCraft](https://github.com/storytold/photocraft), with a static, searchable storefront powered directly by [`catalog.json`](catalog.json).

## Store website

The website lives in [`index.html`](index.html) and has no build-time JavaScript dependencies. It reads `catalog.json` at runtime, so adding a valid plugin entry automatically adds a card to the catalog.

To publish it:

1. Open **Settings → Pages** in this repository.
2. Under **Build and deployment**, select **GitHub Actions** as the source.
3. Push to `main` or manually run **Deploy plugin store website** from the Actions tab.
4. Open the URL shown in the workflow's deployment environment.

The workflow validates `catalog.json` and deploys the site using GitHub Pages.

## Available plug-ins

| Plug-in | Type | What it does |
|---|---|---|
| **Smudge Blend** | Filter (ABI v1) | Smears pixels in a chosen direction with adjustable strength and radius. |
| **Mixer Blend** | Filter (ABI v1) | Mixes nearby colours with adjustable pickup, radius, and wetness. |

### Important compatibility note

PhotoCraft's current public plug-in ABI v1 supports **whole-buffer filters only**. It does not expose pointer/mouse-drag events, brush cursors, or persistent per-stroke state. Therefore these entries are smudge-/mixer-inspired filters, **not interactive brush tools** like Photoshop's brushes. A true drag-to-paint brush requires changes to PhotoCraft's core tool system.

## Install

1. Open a plug-in's **Download** link (it points to the latest GitHub Release asset).
2. In a compatible PhotoCraft build, choose **Filter → Plug-ins → Install Plug-in…** and select the downloaded `.wasm` file.
3. The filter appears under **Filter → Plug-ins**.

If a release has not been published yet, its download link will not resolve until the first release containing the named asset is available.

The repository's GitHub Actions workflow builds both plug-ins for `wasm32-unknown-unknown` and publishes the `.wasm` files when a version tag (for example, `v0.1.0`) is pushed.

## Catalog schema

Each plugin entry should provide a stable `id`, `name`, `version`, `author`, `description`, `kind`, `abi`, `source`, and `tags`. Use `interactiveBrush: true` only for genuinely interactive tools supported by the host API. Optional `artifact` and `releaseAsset` fields identify a downloadable release asset.

## Build locally

Requirements: Rust stable and `rustup`.

```sh
rustup target add wasm32-unknown-unknown
bash build-all.sh
```

Built modules are copied to `dist/`.

## License

MIT. Community project; not affiliated with the PhotoCraft maintainers.
