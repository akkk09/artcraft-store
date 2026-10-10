# ArtCraft Store API v1 — Agent Reference

This document is the implementation-oriented reference for AI agents, coding assistants, and third-party launcher developers integrating the ArtCraft Store catalog.

## Canonical resources

- Discovery manifest: https://akkk09.github.io/artcraft-store/api/v1/manifest.json
- Catalog (source of truth for listings): https://akkk09.github.io/artcraft-store/catalog.json
- JavaScript ES module: https://akkk09.github.io/artcraft-store/api/v1/client.js
- Human-readable project README: https://github.com/akkk09/artcraft-store#readme

API version: `v1`  
Transport: HTTPS, static JSON and JavaScript module  
Authentication: none  
Mutability: public, read-only  
Server-side query parameters: none

## Integration instructions for agents

1. Fetch the discovery manifest and catalog from the canonical URLs above.
2. Parse `catalog.json` as JSON. The top-level `plugins` property is an array of listings; `apps` may describe supported applications.
3. Filter listings locally by `app`, `kind`, or other fields your launcher understands.
4. Identify an item by its `id` field. Treat IDs as stable lookup keys, but handle missing IDs and duplicate or malformed data defensively.
5. Resolve the download URL using the precedence rules below.
6. Before installing, independently validate the target app, version compatibility, artifact type, source/trust policy, and the actual HTTP download response.
7. Ignore unknown fields for forward compatibility. Do not assume that a field absent from a listing has a default value.

Do not infer installation support from catalog presence. This API discovers metadata and resolves URLs; it does not install, validate, sandbox, or execute plugins.

## Catalog schema

The catalog is JSON. The current client validates that `plugins` is an array, but launchers should validate all fields they depend on.

Common plugin fields:

| Field | Meaning | Consumer guidance |
|---|---|---|
| `id` | Listing identifier | Use for lookup; treat as an opaque string. |
| `app` | Target ArtCraft app ID | Known IDs: `photocraft`, `effectcraft`, `vectorcraft`, `filmcraft`, `soundcraft`, `pdfcraft`, `designcraft`, `lightcraft`, `cadcraft`. |
| `name` | Display name | Treat as untrusted display text. |
| `version` | Listing/plugin version | Do not assume it follows a particular versioning scheme unless your installer defines one. |
| `author` | Author/maintainer label | Informational metadata, not identity verification. |
| `description` | Human-readable summary | Treat as untrusted text. |
| `kind` | Listing category/type | Optional filter; values are catalog-defined. |
| `tags` | Search/category labels | Optional; do not assume present. |
| `compatibility` | Compatibility notes or constraints | Interpret according to the value's actual shape; do not assume compatibility if absent. |
| `sourceUrl` | Project/source page | Optional; does not guarantee the source is safe or available. |
| `downloadUrl` | Explicit download URL | Highest URL-resolution priority. |
| `releaseAsset` | Release asset filename/path | Used when `downloadUrl` is absent. |
| `artifact` | Artifact filename/path | Fallback when both higher-priority fields are absent. |

Fields may be absent and new fields may be added. Do not reject a listing solely because it contains unknown fields. Do reject or skip records that fail the minimum validation your launcher requires.

## Download URL resolution

Use the first available non-empty field in this order:

1. `downloadUrl`
2. `releaseAsset`
3. `artifact`

If `downloadUrl` exists, resolve it as a URL relative to the site root (absolute URLs remain absolute). Otherwise, resolve `releaseAsset` or `artifact` as a filename/path under the store's `downloads/` directory.

Canonical download base: https://akkk09.github.io/artcraft-store/downloads/

Important:
- A resolved URL does **not** prove the resource exists or is downloadable.
- A URL does **not** prove an artifact is compatible, trusted, or safe.
- Do not execute an artifact just because the catalog lists it.
- Apply your launcher's own URL scheme policy (for example, allow HTTPS only unless a documented use case requires otherwise).
- Avoid unsafe path concatenation. Use a URL parser and encode/validate relative artifact names according to your platform's requirements.
- The JavaScript helper uses `new URL()` and encodes the artifact value as one path segment. If a catalog value contains a path with intentional subdirectories, confirm that behavior fits the intended artifact layout before relying on it.

## JavaScript client

The dependency-free ES module is available at:

`https://akkk09.github.io/artcraft-store/api/v1/client.js`

It supports modern browser/Electron environments and modern Node.js ESM environments with `fetch`.

### Example

```js
import { artcraft } from "https://akkk09.github.io/artcraft-store/api/v1/client.js";

const catalog = await artcraft.getCatalog();
const photoCraftPlugins = await artcraft.listPlugins({ app: "photocraft" });
const effectCraftPlugins = await artcraft.listPlugins({ app: "effectcraft" });
const plugin = await artcraft.getPlugin("org.photocraft.community.vignette");
const downloadUrl = artcraft.getDownloadUrl(plugin);

console.log(catalog.plugins.length, photoCraftPlugins, effectCraftPlugins, downloadUrl);
```

### Methods

- `getCatalog({ signal? })` → `Promise<Catalog>`
  Fetches and parses the catalog. Throws on non-success HTTP responses, JSON parse errors, or a missing/non-array `plugins` property. Accepts an optional `AbortSignal`.
- `listPlugins({ app?, kind?, signal? } = {})` → `Promise<Plugin[]>`
  Fetches the catalog and filters locally. Filters use exact string equality. If a filter is omitted or falsy, it is not applied.
- `getPlugin(id, options = {})` → `Promise<Plugin | null>`
  Fetches the catalog and returns the first exact `id` match, or `null` if none matches. `options` may contain `signal`.
- `getDownloadUrl(plugin)` → `string | null`
  Resolves the URL using the precedence above. Returns `null` for a null plugin or when no artifact field is configured. It does not make a network request to verify the URL.
- `createArtCraftClient({ baseUrl? } = {})` → client object
  Creates a client rooted at the supplied site URL; defaults to the published site root. Useful for testing a mirror. The root should be a valid URL and should normally end in `/`.

The returned client object is frozen. The module also exports the default singleton `artcraft`.

### Error handling

Network failures, aborts, HTTP failures, and invalid catalog JSON reject the fetch methods' promises. Catch errors at the integration boundary and present a useful failure state; do not silently treat fetch failure as an empty catalog.

```js
try {
  const plugins = await artcraft.listPlugins({ app: "photocraft" });
  // Validate compatibility and installation policy before offering installation.
} catch (error) {
  console.error("Could not load ArtCraft Store catalog", error);
}
```

## Language-agnostic HTTP example

The API is static JSON, so any language with an HTTPS client and JSON parser can integrate without the JavaScript module.

```sh
curl --fail --show-error --location \
  https://akkk09.github.io/artcraft-store/catalog.json
```

Example Python:

```python
import json
from urllib.request import urlopen

CATALOG_URL = "https://akkk09.github.io/artcraft-store/catalog.json"

with urlopen(CATALOG_URL, timeout=10) as response:
    catalog = json.load(response)

plugins = catalog.get("plugins")
if not isinstance(plugins, list):
    raise ValueError("Unexpected ArtCraft catalog format")

for plugin in plugins:
    if isinstance(plugin, dict) and plugin.get("app") == "photocraft":
        print(plugin.get("id"), plugin.get("name"))
```

## Static API behavior and limitations

- There is no backend query endpoint. Fetch the catalog and filter client-side.
- The API is read-only and does not require credentials.
- Do not assume rate limits, uptime guarantees, a particular cache duration, or that every configured artifact is currently available.
- A listing may point to an external host.
- The catalog and helper module may be updated as the store changes. Be tolerant of additional fields.
- This is a community-maintained project, not an official ArtCraft support channel.
- Installing plugins is out of scope. Each launcher must implement app-specific packaging, compatibility checks, safe download handling, installation, rollback, and user consent.

## Security checklist for launcher implementers

- Use HTTPS and validate resolved URL schemes/hosts against your launcher policy.
- Treat catalog strings, descriptions, names, URLs, and artifacts as untrusted input.
- Validate that the requested `app` is supported by your launcher.
- Check compatibility against the installed app and plugin API before installation.
- Verify artifact type, size, and integrity where possible; the catalog does not currently guarantee checksums or signatures.
- Do not execute downloaded code automatically without an explicit, informed user action and your application's normal trust checks.
- Handle missing fields, malformed records, HTTP errors, timeouts, and cancellation.
- Do not hardcode the current catalog contents as permanent API behavior.

## Versioning

The discovery manifest declares `apiVersion: "v1"`. Treat this as the documented API generation, not as a promise that every catalog field is immutable. Clients should ignore unknown fields and depend only on the fields they use. Check the manifest and this document for updated integration guidance.
