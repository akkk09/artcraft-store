#!/usr/bin/env python3
"""Probe every configured catalog download URL after the Pages deployment."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urljoin
from urllib.request import Request, urlopen


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-url", required=True, help="deployed Pages site URL")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    catalog_path = root / "catalog.json"
    try:
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"ERROR: cannot read catalog.json: {exc}")
        return 1

    base_url = args.base_url.rstrip("/") + "/"
    failures: list[str] = []
    checked = 0

    for plugin in catalog.get("plugins", []):
        if not isinstance(plugin, dict):
            continue

        configured_url = plugin.get("downloadUrl")
        if configured_url:
            url = configured_url
        else:
            artifact = plugin.get("releaseAsset") or plugin.get("artifact")
            if not isinstance(artifact, str) or not artifact:
                continue
            url = urljoin(base_url, "downloads/" + quote(artifact, safe=""))

        checked += 1
        try:
            request = Request(url, headers={"Range": "bytes=0-0"})
            with urlopen(request, timeout=20) as response:
                if response.status < 200 or response.status >= 300:
                    failures.append(f"{plugin.get('name', plugin.get('id', 'item'))}: HTTP {response.status} ({url})")
                else:
                    print(f"OK {response.status} {url}")
        except (HTTPError, URLError, TimeoutError, OSError) as exc:
            failures.append(f"{plugin.get('name', plugin.get('id', 'item'))}: {exc} ({url})")

    if failures:
        for failure in failures:
            print(f"ERROR {failure}")
        print(f"Download audit failed: {len(failures)} of {checked} URLs failed.")
        return 1

    print(f"Download audit passed: {checked} catalog download URLs responded successfully.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
