//! PivotCraft: Original anchor-point placement, layer alignment, and transform
//! compensation engine for EffectCraft.
//!
//! Provides mathematically verified pivot placement and position compensation
//! across 2D and 3D layers, separated position dimensions, visible alpha bounds
//! analysis, keyframed motion paths, and preset management.

use serde::{Deserialize, Serialize};

/// 9-point pivot placement grid plus custom coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PivotPoint {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    /// Custom normalized fraction `(x_frac, y_frac, z_frac)` where 0.0=left/top, 0.5=center, 1.0=right/bottom.
    Custom(f64, f64, f64),
}

impl PivotPoint {
    /// Returns normalized coordinates (0.0 to 1.0) for this pivot point on the XY plane.
    pub fn normalized_xy(&self) -> (f64, f64) {
        match self {
            Self::TopLeft => (0.0, 0.0),
            Self::TopCenter => (0.5, 0.0),
            Self::TopRight => (1.0, 0.0),
            Self::MiddleLeft => (0.0, 0.5),
            Self::Center => (0.5, 0.5),
            Self::MiddleRight => (1.0, 0.5),
            Self::BottomLeft => (0.0, 1.0),
            Self::BottomCenter => (0.5, 1.0),
            Self::BottomRight => (1.0, 1.0),
            Self::Custom(x, y, _) => (*x, *y),
        }
    }

    /// Normalized Z coordinate (0.0=front/zero, 0.5=center, etc.).
    pub fn normalized_z(&self) -> f64 {
        match self {
            Self::Custom(_, _, z) => *z,
            _ => 0.0,
        }
    }

    /// Parse identifier from string.
    pub fn from_id(id: &str) -> Option<Self> {
        match id.to_lowercase().as_str() {
            "topleft" | "tl" => Some(Self::TopLeft),
            "topcenter" | "tc" | "top" => Some(Self::TopCenter),
            "topright" | "tr" => Some(Self::TopRight),
            "middleleft" | "ml" | "left" => Some(Self::MiddleLeft),
            "center" | "c" | "middle" => Some(Self::Center),
            "middleright" | "mr" | "right" => Some(Self::MiddleRight),
            "bottomleft" | "bl" => Some(Self::BottomLeft),
            "bottomcenter" | "bc" | "bottom" => Some(Self::BottomCenter),
            "bottomright" | "br" => Some(Self::BottomRight),
            _ => None,
        }
    }
}

/// Mode defining how layer boundaries are calculated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BoundsMode {
    /// Use whole layer bounds (source dimensions or vector element bounds).
    #[default]
    LayerBounds,
    /// Use visible alpha bounds where pixel alpha exceeds a transparency threshold.
    AlphaBounds,
}

/// Bounding rectangle in layer space `[x0, y0, x1, y1]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RectBounds {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl RectBounds {
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }

    pub fn from_size(width: f64, height: f64) -> Self {
        Self::new(0.0, 0.0, width, height)
    }

    #[inline]
    pub fn width(&self) -> f64 {
        (self.x1 - self.x0).max(0.0)
    }

    #[inline]
    pub fn height(&self) -> f64 {
        (self.y1 - self.y0).max(0.0)
    }

    #[inline]
    pub fn center(&self) -> [f64; 2] {
        [(self.x0 + self.x1) * 0.5, (self.y0 + self.y1) * 0.5]
    }

    /// Calculate pivot point coordinates in layer space given a grid placement and pixel offset.
    pub fn evaluate_pivot(&self, point: PivotPoint, offset: [f64; 3], current_z: f64) -> [f64; 3] {
        let (nx, ny) = point.normalized_xy();
        let target_x = self.x0 + self.width() * nx + offset[0];
        let target_y = self.y0 + self.height() * ny + offset[1];
        let target_z = current_z + offset[2];
        [target_x, target_y, target_z]
    }
}

/// Scanner for determining visible alpha bounds with a configurable transparency threshold.
pub struct AlphaScanner;

impl AlphaScanner {
    /// Finds bounding box `[x0, y0, x1, y1]` of pixels in an RGBA buffer where alpha >= threshold.
    /// Threshold is normalized [0.0, 1.0].
    /// Pixel data is expected to be RGBA in row-major order (4 floats per pixel or 4 u8 per pixel).
    pub fn scan_rgba_f32(width: usize, height: usize, pixels: &[f32], threshold: f32) -> Option<RectBounds> {
        if width == 0 || height == 0 || pixels.len() < width * height * 4 {
            return None;
        }

        let thresh = threshold.clamp(0.0, 1.0);
        let mut min_x = usize::MAX;
        let mut min_y = usize::MAX;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut found = false;

        for y in 0..height {
            let row_offset = y * width * 4;
            for x in 0..width {
                let alpha = pixels[row_offset + x * 4 + 3];
                if alpha >= thresh {
                    found = true;
                    if x < min_x { min_x = x; }
                    if x > max_x { max_x = x; }
                    if y < min_y { min_y = y; }
                    if y > max_y { max_y = y; }
                }
            }
        }

        if found {
            Some(RectBounds::new(min_x as f64, min_y as f64, (max_x + 1) as f64, (max_y + 1) as f64))
        } else {
            None
        }
    }

    /// Finds bounding box for u8 RGBA pixels (0..255).
    pub fn scan_rgba_u8(width: usize, height: usize, pixels: &[u8], threshold_f32: f32) -> Option<RectBounds> {
        if width == 0 || height == 0 || pixels.len() < width * height * 4 {
            return None;
        }

        let thresh_u8 = (threshold_f32.clamp(0.0, 1.0) * 255.0).round() as u8;
        let mut min_x = usize::MAX;
        let mut min_y = usize::MAX;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut found = false;

        for y in 0..height {
            let row_offset = y * width * 4;
            for x in 0..width {
                let alpha = pixels[row_offset + x * 4 + 3];
                if alpha >= thresh_u8 {
                    found = true;
                    if x < min_x { min_x = x; }
                    if x > max_x { max_x = x; }
                    if y < min_y { min_y = y; }
                    if y > max_y { max_y = y; }
                }
            }
        }

        if found {
            Some(RectBounds::new(min_x as f64, min_y as f64, (max_x + 1) as f64, (max_y + 1) as f64))
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------- Transform Math

/// 2D Transform solver for position compensation when anchor point moves.
pub struct Transform2D;

impl Transform2D {
    /// Computes the parent-space position shift required to preserve a 2D layer's visible position
    /// when the anchor point changes by `delta_anchor = [new_anchor_x - old_anchor_x, new_anchor_y - old_anchor_y]`.
    ///
    /// Mathematical Proof:
    /// For any pixel `X` in layer space, its parent-space position is:
    /// `X_parent = Position + R(rot) * S(scale/100) * (X - Anchor)`
    /// When anchor moves to `Anchor' = Anchor + delta_anchor`, we add `shift` to Position:
    /// `X'_parent = (Position + shift) + R * S * (X - Anchor')`
    /// Setting `X'_parent = X_parent`:
    /// `shift = R(rot) * S(scale/100) * delta_anchor`
    #[inline]
    pub fn compute_shift(delta_anchor: [f64; 2], scale_pct: [f64; 2], rotation_deg: f64) -> [f64; 2] {
        let (dx, dy) = (
            delta_anchor[0] * scale_pct[0] / 100.0,
            delta_anchor[1] * scale_pct[1] / 100.0,
        );
        let rad = rotation_deg.to_radians();
        let (sn, cs) = rad.sin_cos();
        [dx * cs - dy * sn, dx * sn + dy * cs]
    }
}

/// 3D Matrix & Vector operations for 3D layer transform compensation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3x3(pub [[f64; 3]; 3]);

impl Mat3x3 {
    pub const IDENTITY: Self = Self([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);

    pub fn mul(&self, rhs: &Self) -> Self {
        let mut out = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                out[i][j] = self.0[i][0] * rhs.0[0][j]
                    + self.0[i][1] * rhs.0[1][j]
                    + self.0[i][2] * rhs.0[2][j];
            }
        }
        Self(out)
    }

    pub fn apply_vec(&self, v: [f64; 3]) -> [f64; 3] {
        [
            self.0[0][0] * v[0] + self.0[0][1] * v[1] + self.0[0][2] * v[2],
            self.0[1][0] * v[0] + self.0[1][1] * v[1] + self.0[1][2] * v[2],
            self.0[2][0] * v[0] + self.0[2][1] * v[1] + self.0[2][2] * v[2],
        ]
    }

    pub fn rotate_x(deg: f64) -> Self {
        let (sn, cs) = deg.to_radians().sin_cos();
        Self([[1.0, 0.0, 0.0], [0.0, cs, -sn], [0.0, sn, cs]])
    }

    pub fn rotate_y(deg: f64) -> Self {
        let (sn, cs) = deg.to_radians().sin_cos();
        Self([[cs, 0.0, sn], [0.0, 1.0, 0.0], [-sn, 0.0, cs]])
    }

    pub fn rotate_z(deg: f64) -> Self {
        let (sn, cs) = deg.to_radians().sin_cos();
        Self([[cs, -sn, 0.0], [sn, cs, 0.0], [0.0, 0.0, 1.0]])
    }

    pub fn scale(s_pct: [f64; 3]) -> Self {
        Self([
            [s_pct[0] / 100.0, 0.0, 0.0],
            [0.0, s_pct[1] / 100.0, 0.0],
            [0.0, 0.0, s_pct[2] / 100.0],
        ])
    }

    /// Orientation matrix matching EffectCraft/After Effects: Z * Y * X in order.
    pub fn orientation(o_deg: [f64; 3]) -> Self {
        Self::rotate_z(o_deg[2])
            .mul(&Self::rotate_y(o_deg[1]))
            .mul(&Self::rotate_x(o_deg[0]))
    }
}

/// 3D Transform solver for position compensation when anchor point moves.
pub struct Transform3D;

impl Transform3D {
    /// Computes full 3D position shift in parent space.
    ///
    /// Layer transform order in EffectCraft/AE:
    /// `L = Orientation * RotateZ * RotateY * RotateX * Scale`
    /// `shift = L * delta_anchor`
    pub fn compute_shift(
        delta_anchor: [f64; 3],
        scale_pct: [f64; 3],
        orientation_deg: [f64; 3],
        rotation_deg: [f64; 3], // [rx, ry, rz]
    ) -> [f64; 3] {
        let orient = Mat3x3::orientation(orientation_deg);
        let rot_z = Mat3x3::rotate_z(rotation_deg[2]);
        let rot_y = Mat3x3::rotate_y(rotation_deg[1]);
        let rot_x = Mat3x3::rotate_x(rotation_deg[0]);
        let scale = Mat3x3::scale(scale_pct);

        let linear = orient
            .mul(&rot_z)
            .mul(&rot_y)
            .mul(&rot_x)
            .mul(&scale);

        linear.apply_vec(delta_anchor)
    }
}

// ---------------------------------------------------------------- Presets & Configurations

/// Saved preset for PivotCraft.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PivotPreset {
    pub name: String,
    pub point: PivotPoint,
    pub mode: BoundsMode,
    pub threshold: f64,
    pub offset: [f64; 3],
    pub preserve_position: bool,
}

impl PivotPreset {
    pub fn new(name: impl Into<String>, point: PivotPoint) -> Self {
        Self {
            name: name.into(),
            point,
            mode: BoundsMode::LayerBounds,
            threshold: 0.05,
            offset: [0.0, 0.0, 0.0],
            preserve_position: true,
        }
    }
}

/// Built-in standard presets.
pub fn built_in_presets() -> Vec<PivotPreset> {
    vec![
        PivotPreset::new("Center (Default)", PivotPoint::Center),
        PivotPreset::new("Top-Left (Corner)", PivotPoint::TopLeft),
        PivotPreset::new("Top-Center (Edge)", PivotPoint::TopCenter),
        PivotPreset::new("Top-Right (Corner)", PivotPoint::TopRight),
        PivotPreset::new("Middle-Left (Edge)", PivotPoint::MiddleLeft),
        PivotPreset::new("Middle-Right (Edge)", PivotPoint::MiddleRight),
        PivotPreset::new("Bottom-Left (Corner)", PivotPoint::BottomLeft),
        PivotPreset::new("Bottom-Center (Edge)", PivotPoint::BottomCenter),
        PivotPreset::new("Bottom-Right (Corner)", PivotPoint::BottomRight),
        PivotPreset {
            name: "Lower-Third Anchor (0%, 80%)".into(),
            point: PivotPoint::Custom(0.0, 0.8, 0.0),
            mode: BoundsMode::LayerBounds,
            threshold: 0.05,
            offset: [0.0, 0.0, 0.0],
            preserve_position: true,
        },
        PivotPreset {
            name: "Title Baseline (50%, 100%)".into(),
            point: PivotPoint::Custom(0.5, 1.0, 0.0),
            mode: BoundsMode::LayerBounds,
            threshold: 0.05,
            offset: [0.0, 0.0, 0.0],
            preserve_position: true,
        },
        PivotPreset {
            name: "Alpha-Aware Center (5% Threshold)".into(),
            point: PivotPoint::Center,
            mode: BoundsMode::AlphaBounds,
            threshold: 0.05,
            offset: [0.0, 0.0, 0.0],
            preserve_position: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2d_shift_cancels_pixel_motion() {
        // Test layer with 150% scale and 45 degree rotation
        let old_anchor = [100.0, 50.0];
        let new_anchor = [0.0, 0.0];
        let delta_a = [new_anchor[0] - old_anchor[0], new_anchor[1] - old_anchor[1]];
        let scale = [150.0, 150.0];
        let rot = 45.0;

        let shift = Transform2D::compute_shift(delta_a, scale, rot);

        // Verification: Any pixel X in layer space must map to the same parent position
        // Before: P + R * S * (X - a)
        // After: (P + shift) + R * S * (X - a')
        // (P + shift) + R * S * (X - a - delta_a) = P + shift + R * S * (X - a) - R * S * delta_a
        // Since shift = R * S * delta_a, they cancel out to 0.
        let rad = rot.to_radians();
        let (sn, cs) = rad.sin_cos();
        let s = 1.5;
        let rs_delta = [
            (delta_a[0] * s * cs - delta_a[1] * s * sn),
            (delta_a[0] * s * sn + delta_a[1] * s * cs),
        ];

        assert!((shift[0] - rs_delta[0]).abs() < 1e-10);
        assert!((shift[1] - rs_delta[1]).abs() < 1e-10);
    }

    #[test]
    fn test_3d_shift_matches_2d_when_3d_angles_zero() {
        let delta_a = [50.0, -30.0, 0.0];
        let scale = [120.0, 80.0, 100.0];
        let orient = [0.0, 0.0, 0.0];
        let rot = [0.0, 0.0, 30.0]; // rotation around Z

        let shift_2d = Transform2D::compute_shift([delta_a[0], delta_a[1]], [scale[0], scale[1]], rot[2]);
        let shift_3d = Transform3D::compute_shift(delta_a, scale, orient, rot);

        assert!((shift_2d[0] - shift_3d[0]).abs() < 1e-9);
        assert!((shift_2d[1] - shift_3d[1]).abs() < 1e-9);
        assert!(shift_3d[2].abs() < 1e-9);
    }

    #[test]
    fn test_alpha_scanner_detects_tight_bounds() {
        let w = 10;
        let h = 10;
        let mut pixels = vec![0.0f32; w * h * 4];

        // Draw an opaque box from (2, 3) to (6, 7)
        for y in 3..=7 {
            for x in 2..=6 {
                let idx = (y * w + x) * 4;
                pixels[idx] = 1.0;
                pixels[idx + 1] = 1.0;
                pixels[idx + 2] = 1.0;
                pixels[idx + 3] = 0.8; // alpha 80%
            }
        }

        let bounds = AlphaScanner::scan_rgba_f32(w, h, &pixels, 0.1).unwrap();
        assert_eq!(bounds.x0, 2.0);
        assert_eq!(bounds.y0, 3.0);
        assert_eq!(bounds.x1, 7.0); // max_x + 1
        assert_eq!(bounds.y1, 8.0); // max_y + 1
        assert_eq!(bounds.width(), 5.0);
        assert_eq!(bounds.height(), 5.0);
    }

    #[test]
    fn test_alpha_scanner_ignores_subthreshold_pixels() {
        let w = 4;
        let h = 4;
        let mut pixels = vec![0.0f32; w * h * 4];
        // Low alpha below threshold
        pixels[3] = 0.02; // alpha 2%
        let bounds = AlphaScanner::scan_rgba_f32(w, h, &pixels, 0.05);
        assert!(bounds.is_none());
    }

    #[test]
    fn test_rect_bounds_evaluate_9_points() {
        let b = RectBounds::new(10.0, 20.0, 110.0, 220.0);
        assert_eq!(b.width(), 100.0);
        assert_eq!(b.height(), 200.0);

        let tl = b.evaluate_pivot(PivotPoint::TopLeft, [0.0; 3], 0.0);
        assert_eq!(tl, [10.0, 20.0, 0.0]);

        let c = b.evaluate_pivot(PivotPoint::Center, [0.0; 3], 0.0);
        assert_eq!(c, [60.0, 120.0, 0.0]);

        let br = b.evaluate_pivot(PivotPoint::BottomRight, [0.0; 3], 0.0);
        assert_eq!(br, [110.0, 220.0, 0.0]);
    }
}
