# ArtCraft Store

A shared, searchable community catalog for extensions across the ArtCraft creative apps:

- **PhotoCraft** — plug-ins, filters, and tools
- **FilmCraft** — plug-ins, effects, and presets
- **EffectCraft** — scripts and plug-ins
- **VectorCraft** — plug-ins, tools, and presets

The static storefront in [`index.html`](index.html) reads [`catalog.json`](catalog.json) at runtime. Filter by app or search across names, descriptions, tags, and compatibility notes.

This is a community-maintained catalog, not an official ArtCraft support channel. Submissions should be limited to plugins and scripts intended for listing in this store, with a source repository, license, download link, supported app/version, and clear description. Reviews are for existing catalog items only.

## Launcher API (v1)

Third-party launchers can use the public, read-only static API. It requires no API key, account, or backend service.

- **API discovery manifest:** https://akkk09.github.io/artcraft-store/api/v1/manifest.json
- **Catalog JSON:** https://akkk09.github.io/artcraft-store/catalog.json
- **Client module:** https://akkk09.github.io/artcraft-store/api/v1/client.js

The catalog endpoint returns JSON with `apps` and `plugins` arrays. Plugin IDs are stable lookup keys; clients should ignore unknown fields so new metadata can be added without breaking older launchers. The manifest documents how download URLs are resolved. This is a static API, so filtering and ID lookup happen client-side; it is not a server-side query API.

### Use the module

In a browser or Electron launcher using JavaScript modules:

```js
import { artcraft } from "https://akkk09.github.io/artcraft-store/api/v1/client.js";

const plugins = await artcraft.listPlugins({ app: "effectcraft" });
const plugin = await artcraft.getPlugin("org.effectcraft.trokute.chromatic-fringe");
const downloadUrl = artcraft.getDownloadUrl(plugin);

console.log(plugins, downloadUrl);
```

You can also import `createArtCraftClient` and pass a different `baseUrl` when testing a mirror:

```js
import { createArtCraftClient } from "https://akkk09.github.io/artcraft-store/api/v1/client.js";

const store = createArtCraftClient({ baseUrl: "https://example.com/artcraft-store/" });
const catalog = await store.getCatalog();
```

### Use the endpoint from any language

Every launcher can use ordinary HTTP and JSON without adopting the JavaScript module:

```sh
curl -fsSL https://akkk09.github.io/artcraft-store/catalog.json
```

Python example:

```python
import json
from urllib.request import urlopen

url = "https://akkk09.github.io/artcraft-store/catalog.json"
with urlopen(url, timeout=10) as response:
    catalog = json.load(response)

for plugin in catalog["plugins"]:
    print(plugin["id"], plugin["name"], plugin.get("app"))
```

The client exposes `getCatalog()`, `listPlugins({ app, kind })`, `getPlugin(id)`, and `getDownloadUrl(plugin)`. `getPlugin` returns `null` when no ID matches; `getDownloadUrl` returns `null` when a listing has no configured artifact. Callers should handle network failures and validate compatibility before installing files.

### Endpoint limitations

This API is intentionally static and read-only. It does not install plugins, guarantee that every listed file is available, or provide server-side filtering, authentication, or rate-limit guarantees. Direct download URLs are resolved from `downloadUrl` when present, otherwise from `releaseAsset` or `artifact` under the store's `downloads/` directory. A catalog entry alone does not prove its download file exists.

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
| **Gradient Map & Duotone** | Filter (ABI v1) | Maps luminance to one of five two-colour gradients with adjustable intensity. |

The current build workflow only compiles the PhotoCraft Rust/WASM plug-ins. FilmCraft, EffectCraft, and VectorCraft listings can use their own source repositories and download URLs; their native build/install workflows should be integrated separately once the relevant APIs and formats are confirmed. No unverified listings are fabricated for those apps.

## Validate the catalog

Run the dependency-free catalog validator and its tests with Python 3:

```sh
python scripts/validate_catalog.py
python -m unittest discover -s tests -v
```

## Build PhotoCraft plug-ins locally

Requirements: Rust stable and `rustup`.

```sh
rustup target add wasm32-unknown-unknown
bash build-all.sh
```

Built modules are copied to `dist/`.

## License

MIT. Community project; not affiliated with the ArtCraft app maintainers.
