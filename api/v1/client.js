const DEFAULT_SITE_ROOT = new URL("../../", import.meta.url);

/**
 * Tiny dependency-free client for the public ArtCraft Store catalog.
 * Works in browser/Electron launchers and modern Node.js ESM projects.
 */
export function createArtCraftClient({ baseUrl = DEFAULT_SITE_ROOT } = {}) {
  const root = new URL(baseUrl);
  const catalogUrl = new URL("catalog.json", root);

  async function getCatalog({ signal } = {}) {
    const response = await fetch(catalogUrl, { signal });
    if (!response.ok) {
      throw new Error(`ArtCraft catalog request failed: HTTP ${response.status}`);
    }

    const catalog = await response.json();
    if (!catalog || !Array.isArray(catalog.plugins)) {
      throw new Error("ArtCraft catalog response has an invalid format");
    }
    return catalog;
  }

  async function listPlugins({ app, kind, signal } = {}) {
    const catalog = await getCatalog({ signal });
    return catalog.plugins.filter(plugin =>
      (!app || plugin.app === app) &&
      (!kind || plugin.kind === kind)
    );
  }

  async function getPlugin(id, options = {}) {
    const catalog = await getCatalog(options);
    return catalog.plugins.find(plugin => plugin.id === id) ?? null;
  }

  function getDownloadUrl(plugin) {
    if (!plugin) return null;
    if (plugin.downloadUrl) return new URL(plugin.downloadUrl, root).href;

    const artifact = plugin.releaseAsset || plugin.artifact;
    if (!artifact) return null;

    return new URL(`downloads/${encodeURIComponent(artifact)}`, root).href;
  }

  return Object.freeze({
    catalogUrl: catalogUrl.href,
    getCatalog,
    listPlugins,
    getPlugin,
    getDownloadUrl
  });
}

export const artcraft = createArtCraftClient();
