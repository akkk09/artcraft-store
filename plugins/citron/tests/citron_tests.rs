//! Comprehensive unit and integration test suite for Citron Core Curve Engine.

use citron::{
    BufferSnapshot, ChannelCurve, InterpType, KeyPoint, LatticeAnchor, LatticeCage,
    PresetTangents,
};

#[test]
fn test_bezier_evaluation_and_endpoints() {
    let mut ch = ChannelCurve::new("rot", "Rotation", [0.2, 0.6, 1.0, 1.0]);
    let k0 = KeyPoint::with_ease(0.0, 0.0, 33.333, 0.0, 50.0, 0.0);
    let k1 = KeyPoint::with_ease(2.0, 360.0, 50.0, 0.0, 33.333, 0.0);
    ch.keys.push(k0);
    ch.keys.push(k1);

    // Endpoints
    assert_eq!(ch.eval(0.0), 0.0);
    assert_eq!(ch.eval(2.0), 360.0);

    // Midpoint should be smooth progression
    let mid = ch.eval(1.0);
    assert!(mid > 100.0 && mid < 260.0, "Midpoint must be in reasonable range: {}", mid);

    // Monotonic progression for ease in-out
    let mut prev = 0.0;
    for step in 1..=20 {
        let t = (step as f64) * 0.1;
        let v = ch.eval(t);
        assert!(v >= prev, "S-curve should be monotonically increasing at t={}: v={}, prev={}", t, v, prev);
        prev = v;
    }
}

#[test]
fn test_de_casteljau_keyframe_insertion_preserves_trajectory() {
    let mut ch = ChannelCurve::new("pos_y", "Position Y", [0.3, 1.0, 0.3, 1.0]);
    let k0 = KeyPoint::with_ease(0.0, 100.0, 33.333, 0.0, 60.0, 20.0);
    let k1 = KeyPoint::with_ease(2.0, 500.0, 60.0, -10.0, 33.333, 0.0);
    ch.keys.push(k0.clone());
    ch.keys.push(k1.clone());

    // Sample 20 points before split
    let mut before_samples = Vec::new();
    for i in 0..=20 {
        let t = i as f64 * 0.1;
        before_samples.push((t, ch.eval(t)));
    }

    // Insert keyframe at t = 0.85
    let new_idx = ch.insert_key_at(0.85).expect("Insert must succeed");
    assert_eq!(new_idx, 1);
    assert_eq!(ch.keys.len(), 3);

    // Verify samples after split match within small epsilon
    for (t, v_expected) in before_samples {
        let v_actual = ch.eval(t);
        let diff = (v_actual - v_expected).abs();
        assert!(diff < 0.05, "Curve deviated at t={}: expected {}, got {}, diff={}", t, v_expected, v_actual, diff);
    }
}

#[test]
fn test_ffd_lattice_cage_scaling_and_retiming() {
    let mut ch = ChannelCurve::new("scale_x", "Scale X", [1.0, 0.8, 0.2, 1.0]);
    ch.keys.push(KeyPoint::new(1.0, 50.0));
    ch.keys.push(KeyPoint::new(2.0, 100.0));
    ch.keys.push(KeyPoint::new(3.0, 200.0));

    let cage = LatticeCage {
        time_scale: 2.0,   // double duration (stretch)
        value_scale: 0.5,  // half amplitude
        time_offset: 0.0,
        value_offset: 0.0,
        time_skew: 0.0,
        anchor: LatticeAnchor::MinLeft,
    };

    cage.transform_keys(&mut ch.keys);

    // Initial min time was 1.0. With MinLeft anchor and time_scale=2.0:
    // k0 (t=1.0) -> 1.0 + (1-1)*2 = 1.0
    // k1 (t=2.0) -> 1.0 + (2-1)*2 = 3.0
    // k2 (t=3.0) -> 1.0 + (3-1)*2 = 5.0
    assert_eq!(ch.keys[0].time, 1.0);
    assert_eq!(ch.keys[1].time, 3.0);
    assert_eq!(ch.keys[2].time, 5.0);

    // Value with MinLeft anchor (v_min=50.0) and value_scale=0.5:
    // k0 (v=50.0)  -> 50 + (50-50)*0.5 = 50.0
    // k1 (v=100.0) -> 50 + (100-50)*0.5 = 75.0
    // k2 (v=200.0) -> 50 + (200-50)*0.5 = 125.0
    assert_eq!(ch.keys[0].value, 50.0);
    assert_eq!(ch.keys[1].value, 75.0);
    assert_eq!(ch.keys[2].value, 125.0);
}

#[test]
fn test_ffd_lattice_cage_time_skew() {
    let mut keys = vec![
        KeyPoint::new(0.0, 0.0),
        KeyPoint::new(0.0, 100.0),
    ];

    let cage = LatticeCage {
        time_scale: 1.0,
        value_scale: 1.0,
        time_offset: 0.0,
        value_offset: 0.0,
        time_skew: 0.5,
        anchor: LatticeAnchor::Center,
    };

    cage.transform_keys(&mut keys);

    // Top key should skew differently in time than bottom key
    assert_ne!(keys[0].time, keys[1].time, "Vertical time skew must shift keys according to value");
}

#[test]
fn test_buffer_snapshot_rmse() {
    let mut ch = ChannelCurve::new("opacity", "Opacity", [0.8, 0.2, 1.0, 1.0]);
    ch.keys.push(KeyPoint::new(0.0, 0.0));
    ch.keys.push(KeyPoint::new(1.0, 100.0));

    let snap = BufferSnapshot::capture(&[ch.clone()]);
    // RMSE before changes should be 0.0
    assert_eq!(snap.compute_delta_rmse(&[ch.clone()], 50), 0.0);

    // Modify keyframe value
    ch.keys[1].value = 50.0;

    let rmse = snap.compute_delta_rmse(&[ch], 50);
    assert!(rmse > 10.0, "Modified curve must produce non-zero RMSE: {}", rmse);
}

#[test]
fn test_linear_and_hold_interpolation() {
    let mut ch = ChannelCurve::new("stepped", "Hold Channel", [1.0, 1.0, 1.0, 1.0]);
    let mut k0 = KeyPoint::new(0.0, 10.0);
    k0.interp = InterpType::Hold;
    let k1 = KeyPoint::new(2.0, 20.0);
    ch.keys.push(k0);
    ch.keys.push(k1);

    // Hold should stay at 10.0 until exactly 2.0
    assert_eq!(ch.eval(0.5), 10.0);
    assert_eq!(ch.eval(1.99), 10.0);
    assert_eq!(ch.eval(2.0), 20.0);

    // Switch to Linear
    ch.keys[0].interp = InterpType::Linear;
    assert_eq!(ch.eval(1.0), 15.0);
}

#[test]
fn test_preset_tangents_values() {
    let (easy_in, easy_out) = PresetTangents::easy_ease();
    assert_eq!(easy_in.influence, 33.333);
    assert_eq!(easy_in.speed, 0.0);
    assert_eq!(easy_out.influence, 33.333);
    assert_eq!(easy_out.speed, 0.0);

    let (strong_in, strong_out) = PresetTangents::strong_punch();
    assert_eq!(strong_in.influence, 75.0);
    assert_eq!(strong_out.influence, 75.0);

    let (ext_in, ext_out) = PresetTangents::extreme_snap();
    assert_eq!(ext_in.influence, 88.0);
    assert_eq!(ext_out.influence, 88.0);
}

#[test]
fn test_speed_derivative_at_rest() {
    let mut ch = ChannelCurve::new("pos", "Position", [1.0, 0.5, 0.2, 1.0]);
    let k0 = KeyPoint::with_ease(0.0, 100.0, 33.333, 0.0, 33.333, 0.0);
    let k1 = KeyPoint::with_ease(1.0, 100.0, 33.333, 0.0, 33.333, 0.0);
    ch.keys.push(k0);
    ch.keys.push(k1);

    // Flat curve speed must be near zero
    assert!(ch.speed_at(0.5).abs() < 1e-3);
}
