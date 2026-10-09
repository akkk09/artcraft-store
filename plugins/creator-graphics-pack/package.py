#!/usr/bin/env python3
"""Build and validate the distributable Creator Graphics Pack ZIP."""
from pathlib import Path
import argparse
import zipfile

ROOT = Path(__file__).resolve().parents[2]
ASSET_DIR = ROOT / "assets" / "creator-graphics-pack"
EXPECTED = {
    "README.md", "LICENSE.txt", "video-thumbnail.svg", "channel-banner.svg",
    "end-screen.svg", "lower-third.svg", "social-post.svg", "creator-avatar.svg",
}

def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    present = {p.name for p in ASSET_DIR.iterdir() if p.is_file()}
    missing = EXPECTED - present
    if missing:
        raise SystemExit(f"Missing Creator Graphics Pack files: {', '.join(sorted(missing))}")
    for name in EXPECTED:
        if name.endswith(".svg"):
            content = (ASSET_DIR / name).read_text(encoding="utf-8")
            if "<svg" not in content or "</svg>" not in content:
                raise SystemExit(f"Invalid SVG wrapper: {name}")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(args.output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(ASSET_DIR.iterdir()):
            if path.is_file():
                archive.write(path, Path("creator-graphics-pack") / path.name)
    with zipfile.ZipFile(args.output) as archive:
        names = set(archive.namelist())
        expected_names = {str(Path("creator-graphics-pack") / name) for name in EXPECTED}
        if names != expected_names:
            raise SystemExit("ZIP archive contents do not match the expected template set")
    print(f"Packaged {len([n for n in EXPECTED if n.endswith('.svg')])} SVG templates: {args.output}")

if __name__ == "__main__":
    main()
