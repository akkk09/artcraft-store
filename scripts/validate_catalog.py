#!/usr/bin/env python3
"""Validate ArtCraft Store catalog metadata using only the Python standard library."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from urllib.parse import urlparse

APP_IDS = {"photocraft", "filmcraft", "effectcraft", "vectorcraft"}
PLUGIN_ID_RE = re.compile(r"^[A-Za-z0-9._-]{1,64}$")
VERSION_RE = re.compile(
    r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$"
)
REQUIRED_FIELDS = ("id", "name", "version", "author", "description", "kind", "app")


def _is_http_url(value: object) -> bool:
    if not isinstance(value, str):
        return False
    parsed = urlparse(value)
    return parsed.scheme in {"http", "https"} and bool(parsed.netloc)


def validate_catalog(
    catalog: object, root: Path, *, check_artifacts: bool = False
) -> list[str]:
    """Return all catalog validation errors; an empty list means valid metadata."""
    errors: list[str] = []
    if not isinstance(catalog, dict):
        return ["catalog root must be a JSON object"]

    apps = catalog.get("apps")
    if not isinstance(apps, list):
        errors.append("'apps' must be an array")
        apps = []
    declared_apps = {
        app.get("id")
        for app in apps
        if isinstance(app, dict) and isinstance(app.get("id"), str)
    }
    for app_id in sorted(declared_apps - APP_IDS):
        errors.append(f"unknown app declared: {app_id}")
    for app_id in sorted(APP_IDS - declared_apps):
        errors.append(f"missing app declaration: {app_id}")

    plugins = catalog.get("plugins")
    if not isinstance(plugins, list):
        errors.append("'plugins' must be an array")
        return errors

    seen_ids: set[str] = set()
    for index, plugin in enumerate(plugins):
        label = f"plugins[{index}]"
        if not isinstance(plugin, dict):
            errors.append(f"{label} must be an object")
            continue

        for field in REQUIRED_FIELDS:
            if not isinstance(plugin.get(field), str) or not plugin[field].strip():
                errors.append(f"{label}.{field} must be a non-empty string")

        plugin_id = plugin.get("id")
        if isinstance(plugin_id, str) and plugin_id:
            if not PLUGIN_ID_RE.fullmatch(plugin_id):
                errors.append(f"{label}.id contains unsupported characters or is too long")
            if plugin_id in seen_ids:
                errors.append(f"duplicate plugin id: {plugin_id}")
            seen_ids.add(plugin_id)

        version = plugin.get("version")
        if isinstance(version, str) and not VERSION_RE.fullmatch(version):
            errors.append(f"{label}.version must be a semantic version")

        app_id = plugin.get("app")
        if isinstance(app_id, str) and app_id not in APP_IDS:
            errors.append(f"{label}.app is not a supported app id")

        for field in ("sourceUrl", "downloadUrl"):
            value = plugin.get(field)
            if value is not None and not _is_http_url(value):
                errors.append(f"{label}.{field} must be an absolute HTTP(S) URL")

        source = plugin.get("source")
        if source is not None:
            if not isinstance(source, str) or not source.strip():
                errors.append(f"{label}.source must be a non-empty relative path")
            else:
                source_path = Path(source)
                if source_path.is_absolute() or ".." in source_path.parts:
                    errors.append(f"{label}.source must stay inside the repository")
                elif not (root / source_path).exists():
                    errors.append(f"{label}.source does not exist: {source}")

        for field in ("artifact", "releaseAsset"):
            value = plugin.get(field)
            if value is None:
                continue
            if (
                not isinstance(value, str)
                or not value.strip()
                or Path(value).name != value
                or value in {".", ".."}
            ):
                errors.append(f"{label}.{field} must be a plain artifact filename")

        if check_artifacts:
            artifact = plugin.get("downloadUrl")
            if artifact:
                # Absolute downloads are maintained by their external host.
                continue
            artifact = plugin.get("releaseAsset") or plugin.get("artifact")
            if isinstance(artifact, str) and artifact:
                artifact_path = root / "dist" / artifact
                if not artifact_path.is_file():
                    errors.append(
                        f"{label} artifact is missing from dist/: {artifact}"
                    )

    if check_artifacts:
        errors.extend(validate_download_pipeline(catalog, root))

    return errors


def validate_download_pipeline(catalog: dict, root: Path) -> list[str]:
    """Check that catalog artifacts are built, deployed, and published consistently."""
    errors: list[str] = []
    build_path = root / ".github" / "workflows" / "build-plugins.yml"
    deploy_path = root / ".github" / "workflows" / "deploy-pages.yml"

    try:
        build_workflow = build_path.read_text(encoding="utf-8")
    except OSError as exc:
        return [f"cannot read release workflow: {exc}"]
    try:
        deploy_workflow = deploy_path.read_text(encoding="utf-8")
    except OSError as exc:
        return [f"cannot read Pages workflow: {exc}"]

    if "cp dist/* downloads/" not in deploy_workflow:
        errors.append("Pages workflow does not copy built dist/ artifacts into downloads/")

    release_section = build_workflow.split("files: |", 1)
    if len(release_section) != 2:
        errors.append("release workflow is missing its explicit asset file list")
        published_assets: set[str] = set()
    else:
        published_assets = {
            match.group(1)
            for match in re.finditer(
                r"^\\s+dist/([A-Za-z0-9._-]+)\\s*$",
                release_section[1],
                re.MULTILINE,
            )
        }

    for index, plugin in enumerate(catalog.get("plugins", [])):
        if not isinstance(plugin, dict) or plugin.get("downloadUrl"):
            continue
        artifact = plugin.get("releaseAsset") or plugin.get("artifact")
        if not isinstance(artifact, str) or not artifact:
            continue
        if artifact not in published_assets:
            errors.append(
                f"plugins[{index}] artifact is not included in tagged release assets: {artifact}"
            )

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check-artifacts",
        action="store_true",
        help="also require every locally packaged catalog artifact to exist in dist/",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    catalog_path = root / "catalog.json"
    try:
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"ERROR: cannot read {catalog_path.relative_to(root)}: {exc}")
        return 1

    errors = validate_catalog(catalog, root, check_artifacts=args.check_artifacts)
    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1

    count = len(catalog["plugins"])
    suffix = " and local artifacts" if args.check_artifacts else ""
    print(f"Catalog validation passed: {count} plugin entries{suffix}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
