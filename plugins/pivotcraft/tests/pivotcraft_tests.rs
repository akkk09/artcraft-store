//! Integration tests for the PivotCraft Rust engine.

use pivotcraft::*;

#[test]
fn test_2d_transform_invariance() {
    // Arbitrary parameters:
    // Scale 175% X, 80% Y, Rotation 37.5 degrees, old anchor (120, 80), new anchor (10, 20)
    let old_anchor = [120.0, 80.0];
    let new_anchor = [10.0, 20.0];
    let delta_a = [new_anchor[0] - old_anchor[0], new_anchor[1] - old_anchor[1]];
    let scale = [175.0, 80.0];
    let rot = 37.5;
    let pos_old = [500.0, 300.0];

    let shift = Transform2D::compute_shift(delta_a, scale, rot);
    let pos_new = [pos_old[0] + shift[0], pos_old[1] + shift[1]];

    let rad = rot.to_radians();
    let (sn, cs) = rad.sin_cos();
    let map = |p: [f64; 2], a: [f64; 2], pt: [f64; 2]| -> [f64; 2] {
        let dx = (pt[0] - a[0]) * scale[0] / 100.0;
        let dy = (pt[1] - a[1]) * scale[1] / 100.0;
        [p[0] + dx * cs - dy * sn, p[1] + dx * sn + dy * cs]
    };

    // Verify 5 test points in layer space:
    let test_points = [[0.0, 0.0], [50.0, 50.0], [100.0, 200.0], [-30.0, 70.0], [250.0, 150.0]];
    for pt in test_points {
        let before = map(pos_old, old_anchor, pt);
        let after = map(pos_new, new_anchor, pt);
        assert!((before[0] - after[0]).abs() < 1e-10, "X mismatch at {:?}: before={}, after={}", pt, before[0], after[0]);
        assert!((before[1] - after[1]).abs() < 1e-10, "Y mismatch at {:?}: before={}, after={}", pt, before[1], after[1]);
    }
}

#[test]
fn test_3d_transform_invariance() {
    let old_anchor = [100.0, 150.0, 20.0];
    let new_anchor = [30.0, 40.0, -10.0];
    let delta_a = [new_anchor[0] - old_anchor[0], new_anchor[1] - old_anchor[1], new_anchor[2] - old_anchor[2]];

    let scale = [120.0, 110.0, 90.0];
    let orient = [15.0, 25.0, -10.0];
    let rot = [30.0, -45.0, 60.0];
    let pos_old = [960.0, 540.0, 100.0];

    let shift = Transform3D::compute_shift(delta_a, scale, orient, rot);
    let pos_new = [pos_old[0] + shift[0], pos_old[1] + shift[1], pos_old[2] + shift[2]];

    let orient_mat = Mat3x3::orientation(orient);
    let linear = orient_mat
        .mul(&Mat3x3::rotate_z(rot[2]))
        .mul(&Mat3x3::rotate_y(rot[1]))
        .mul(&Mat3x3::rotate_x(rot[0]))
        .mul(&Mat3x3::scale(scale));

    let map3d = |p: [f64; 3], a: [f64; 3], pt: [f64; 3]| -> [f64; 3] {
        let d = [pt[0] - a[0], pt[1] - a[1], pt[2] - a[2]];
        let l = linear.apply_vec(d);
        [p[0] + l[0], p[1] + l[1], p[2] + l[2]]
    };

    let test_points = [
        [0.0, 0.0, 0.0],
        [100.0, 50.0, 10.0],
        [-20.0, 80.0, -50.0],
        [300.0, 200.0, 100.0],
    ];

    for pt in test_points {
        let before = map3d(pos_old, old_anchor, pt);
        let after = map3d(pos_new, new_anchor, pt);
        assert!((before[0] - after[0]).abs() < 1e-9);
        assert!((before[1] - after[1]).abs() < 1e-9);
        assert!((before[2] - after[2]).abs() < 1e-9);
    }
}

#[test]
fn test_9_grid_anchor_positions() {
    let bounds = RectBounds::new(100.0, 200.0, 300.0, 400.0);
    // Width = 200, Height = 200
    assert_eq!(bounds.evaluate_pivot(PivotPoint::TopLeft, [0.0; 3], 0.0), [100.0, 200.0, 0.0]);
    assert_eq!(bounds.evaluate_pivot(PivotPoint::TopCenter, [0.0; 3], 0.0), [200.0, 200.0, 0.0]);
    assert_eq!(bounds.evaluate_pivot(PivotPoint::TopRight, [0.0; 3], 0.0), [300.0, 200.0, 0.0]);
    assert_eq!(bounds.evaluate_pivot(PivotPoint::MiddleLeft, [0.0; 3], 0.0), [100.0, 300.0, 0.0]);
    assert_eq!(bounds.evaluate_pivot(PivotPoint::Center, [0.0; 3], 0.0), [200.0, 300.0, 0.0]);
    assert_eq!(bounds.evaluate_pivot(PivotPoint::MiddleRight, [0.0; 3], 0.0), [300.0, 300.0, 0.0]);
    assert_eq!(bounds.evaluate_pivot(PivotPoint::BottomLeft, [0.0; 3], 0.0), [100.0, 400.0, 0.0]);
    assert_eq!(bounds.evaluate_pivot(PivotPoint::BottomCenter, [0.0; 3], 0.0), [200.0, 400.0, 0.0]);
    assert_eq!(bounds.evaluate_pivot(PivotPoint::BottomRight, [0.0; 3], 0.0), [300.0, 400.0, 0.0]);
}

#[test]
fn test_alpha_bounds_with_threshold() {
    let w = 8;
    let h = 8;
    let mut buf = vec![0.0f32; w * h * 4];

    // Low alpha noise (0.02)
    for i in 0..(w * h) {
        buf[i * 4 + 3] = 0.02;
    }

    // Meaningful visible subject inside (2, 2) to (5, 5) with alpha >= 0.5
    for y in 2..=5 {
        for x in 2..=5 {
            buf[(y * w + x) * 4 + 3] = 0.8;
        }
    }

    // With threshold 0.1, the subject is detected:
    let bounds = AlphaScanner::scan_rgba_f32(w, h, &buf, 0.1).unwrap();
    assert_eq!(bounds.x0, 2.0);
    assert_eq!(bounds.y0, 2.0);
    assert_eq!(bounds.x1, 6.0);
    assert_eq!(bounds.y1, 6.0);

    // With threshold 0.9, nothing passes:
    let empty = AlphaScanner::scan_rgba_f32(w, h, &buf, 0.9);
    assert!(empty.is_none());
}

#[test]
fn test_preset_serialization_roundtrip() {
    let presets = built_in_presets();
    let json_str = serde_json::to_string_pretty(&presets).unwrap();
    let decoded: Vec<PivotPreset> = serde_json::from_str(&json_str).unwrap();

    assert_eq!(presets.len(), decoded.len());
    assert_eq!(presets[0].name, decoded[0].name);
    assert_eq!(presets[0].point, decoded[0].point);
}

#[test]
fn test_zero_delta_yields_zero_shift() {
    let shift = Transform2D::compute_shift([0.0, 0.0], [150.0, 150.0], 90.0);
    assert_eq!(shift, [0.0, 0.0]);

    let shift_3d = Transform3D::compute_shift([0.0, 0.0, 0.0], [150.0, 150.0, 150.0], [10.0, 20.0, 30.0], [45.0, 45.0, 45.0]);
    assert_eq!(shift_3d, [0.0, 0.0, 0.0]);
}
