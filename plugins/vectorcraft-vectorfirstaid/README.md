# Vector FirstAid (VectorCraft Plug-in)

Vector FirstAid is a vector and document cleanup filter for VectorCraft, inspired by Astute Graphics VectorFirstAid.

## Features

- **Unpainted Path Removal**: Eliminates invisible ghost paths that have no fills, no strokes, and no raster imagery.
- **Stray Point Removal**: Prunes single-point orphan anchors left behind by pen tool clicks.
- **Redundant Node Reduction**:
  - Automatically deduplicates coincident adjacent anchors.
  - Removes collinear nodes along straight segments within a configurable angular tolerance (`collinear_threshold`), simplifying vectors without altering silhouette shape.
- **Empty Clipping Mask Cleanup**: Unwraps or purges unused clipping paths and masks with empty children.
- **Empty Group Removal**: Prunes vacant nested groups recursively.
- **Text Cleanup**: Purges empty text paths and trims leading/trailing whitespace.
- **Endpoint Joining**: Automatically closes open subpaths when start and end anchors lie within a configurable threshold.

## ABI

Implements the official VectorCraft Plug-in ABI v1 (`vc_abi_version`, `vc_manifest`, `vc_alloc`, `vc_run`).
