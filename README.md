# ArtCraft Store

A shared, searchable community catalog for apps with supported extension systems:

- **PhotoCraft** — plug-ins, filters, and tools
- **EffectCraft** — plug-ins

The static storefront in [`index.html`](index.html) reads [`catalog.json`](catalog.json) at runtime. Filter by app or search across names, descriptions, tags, and compatibility notes.

This is a community-maintained catalog, not an official ArtCraft support channel. Submissions should be limited to plugins intended for listing in this store, with a source repository, license, download link, supported app/version, and clear description. Reviews are for existing catalog items only.

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

const plugins = await artcraft.listPlugins({ app: "photocraft" });
const plugin = await artcraft.getPlugin("org.photocraft.community.vignette");
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

Add an entry to `catalog.json` under `plugins`. Supported fields include `id`, `app`, `name`, `version`, `author`, `description`, `kind`, `tags`, `compatibility`, `sourceUrl`, `downloadUrl`, `artifact`, and `releaseAsset`. The `app` value should be `photocraft` or `effectcraft`. Only list an app when the extension targets a supported plug-in or scripting interface. Use `sourceUrl` and `downloadUrl` for extensions hosted outside this repository. The storefront supports app-specific formats; it does not assume every item is a WASM file.

Do not add a listing until its source, license, download, and target-app compatibility have been checked.

## Current PhotoCraft plug-ins

| Plug-in | Type | What it does |
|---|---|---|
| **Smudge Blend** | Filter (ABI v1) | Smears pixels in a chosen direction with adjustable strength and radius. |
| **Mixer Blend** | Filter (ABI v1) | Mixes nearby colours with adjustable pickup, radius, and wetness. |
| **Gradient Map & Duotone** | Filter (ABI v1) | Maps luminance to one of five two-colour gradients with adjustable intensity. |
| **Color Toolkit** | Filter (ABI v1) | Adjusts exposure, contrast, and saturation with alpha preservation. |
| **Detail Recovery** | Filter (ABI v1) | Combines lightweight sharpening and spatial noise reduction. |
| **Film Emulation Toolkit** | Filter (ABI v1) | Applies film-inspired color looks with deterministic grain. |
| **Mosaic Pixelate** | Filter (ABI v1) | Creates block-based pixelation with adjustable block size and intensity. |
| **Vignette** | Filter (ABI v1) | Adds adjustable soft edge darkening to draw attention toward the image center. |
| **White Balance** | Filter (ABI v1) | Corrects warm/cool color casts and green/magenta tint. |
| **LUT Studio** | Filter + browser tool | Applies bundled 3D LUTs in PhotoCraft; imports custom `.cube` files and exports image/video frames in the browser studio. |
| **Geometry Fixer** | Filter (ABI v1) | Straightens rotation, adjusts horizontal/vertical perspective, and controls zoom on a fixed canvas. |
| **Seamless Pattern Generator** | Filter (ABI v1) | Reduces texture tiling seams by blending opposing edges with a smooth falloff. |

The build workflow compiles the PhotoCraft Rust/WASM plug-ins and stages the submitted EffectCraft `chromatic-fringe.wat` extension. The browser-based LUT Studio is available at `https://akkk09.github.io/artcraft-store/lut-studio/`. The catalog supports PhotoCraft plug-ins and EffectCraft plug-ins. Chromatic Fringe is the first EffectCraft listing.

## EffectCraft plug-ins

| Plug-in | Type | What it does |
|---|---|---|
| **Chromatic Fringe** | Plugin (API v1) | Offsets red and blue color channels in opposite directions (`chromatic-fringe.wat`). |
| **DepthCraft** | Plugin (API v1 / WASM) | Monocular depth estimation, edge-preserving bilateral filtering, and atmospheric fog generator (`depthcraft.wasm`). |
| **EaseCraft** | Tool (ScriptUI / ExtendScript) | Visual cubic Bézier curve editor, transition presets, and keyframe easing tool (`EaseCraft.jsx`). |
| **CaptionCraft** | Tool (ScriptUI / ExtendScript) | Caption authoring, SRT/VTT subtitle import/export, and typography layout tool (`CaptionCraft.jsx`). |
| **QuietCraft** | Tool (ScriptUI / ExtendScript) | Automated audio silence detection, noise-floor thresholding, and speech derushing tool (`QuietCraft.jsx`). |

In EffectCraft:
- Native / WASM effect plug-ins (**DepthCraft**, **Chromatic Fringe**) can be loaded via **Effect → Load Effect Plug-in** or placed in `~/.config/effectcraft/plugins/`.
- ScriptUI panels (**EaseCraft**, **CaptionCraft**, **QuietCraft**) are installed to `~/.config/effectcraft/Scripts/ScriptUI Panels/` and opened from the **Window** menu.

## Build PhotoCraft plug-ins locally

Requirements: Rust stable and `rustup`.

```sh
rustup target add wasm32-unknown-unknown
bash build-all.sh
```

PhotoCraft WASM modules and the EffectCraft `.wat` extension are copied to `dist/`. Legacy asset-bundle build steps may remain, but those bundles are not listed as supported app extensions in the catalog.

## License

MIT. Community project; not affiliated with the ArtCraft app maintainers.
