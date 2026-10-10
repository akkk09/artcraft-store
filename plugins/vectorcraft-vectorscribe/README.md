# Vector VectorScribe (VectorCraft Plug-in)

Vector VectorScribe is an advanced node and Bézier path editing toolkit for VectorCraft, inspired by Astute Graphics VectorScribe.

## Features

- **Smart Remove**: Removes intermediate anchor points while mathematically recalculating adjacent control handles to retain the path's exact curve silhouette.
- **Extend Path Tool**:
  - Extends open paths beyond their terminal endpoints.
  - Extension modes: `linear` (tangent direction), `arc` (curved circular arc), and `spiral`.
  - Configurable endpoints: `end`, `start`, or `both`.
- **Reposition Point Tool**:
  - Slides anchor points smoothly along the curve path trajectory without altering the underlying geometry.
- **Smooth Points**:
  - Converts sharp corner nodes to smooth tangent nodes within an angular tolerance (`smooth_tolerance`).
- **Retract Handles**:
  - Retracts Bézier handles into the anchor point, converting smooth vertices into sharp angular corners.
- **Reverse Direction**:
  - Reverses the point order and inverts handles for paths and subpaths.

## ABI

Implements the official VectorCraft Plug-in ABI v1 (`vc_abi_version`, `vc_manifest`, `vc_alloc`, `vc_run`).
