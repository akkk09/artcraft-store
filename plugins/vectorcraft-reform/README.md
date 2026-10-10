# Vector Reform (VectorCraft Plug-in)

Vector Reform is an organic path sculpting and reshaping tool for VectorCraft, inspired by Astute Graphics Reform.

## Features

- **Path Reshaping Without Handle Micro-management**: Sculpt entire paths or specific subsegments naturally.
- **Deformation Modes**:
  - `bulge`: Smoothly expands the path outwards along its surface normals.
  - `pinch`: Indents the path inwards along its surface normals.
  - `bend`: Curves the path between its anchor endpoints.
  - `push`: Directional vector displacement.
  - `taper`: Scales path width towards or away from its centerline.
  - `smooth`: Laplacian curvature smoothing to reduce spikes and irregularities.
- **Range & Falloff Control**:
  - `range_start` & `range_end`: Restrict deformation to a specific segment of the path arc length ($0.0$ to $1.0$).
  - `falloff`: Smooth (half-sine), Linear (triangle), or Sharp curves.
- **Tension & Bézier Handle Continuity**:
  - Automatically recalculates incoming and outgoing Bézier handles to preserve tangent smoothness.
- **Endpoint Preservation**: Safeguards endpoints of open paths.

## ABI

Implements the official VectorCraft Plug-in ABI v1 (`vc_abi_version`, `vc_manifest`, `vc_alloc`, `vc_run`).
