use easecraft::{EaseCurve, MIN_INFLUENCE, MAX_INFLUENCE, PresetPack, path_length_3d};

#[test]
fn test_boundary_endpoints_always_exact() {
    let curves = [
        EaseCurve::linear(),
        EaseCurve::ease_in(),
        EaseCurve::ease_out(),
        EaseCurve::ease_in_out(),
        EaseCurve::back_in(),
        EaseCurve::back_out(),
        EaseCurve::back_in_out(),
        EaseCurve::elastic_snap(),
        EaseCurve::bounce_settle(),
    ];

    for c in curves {
        assert_eq!(c.eval(0.0), 0.0, "t=0 must be 0 for {:?}", c);
        assert_eq!(c.eval(1.0), 1.0, "t=1 must be 1 for {:?}", c);
        assert_eq!(c.eval(-0.5), 0.0, "t<0 clamps to 0");
        assert_eq!(c.eval(1.5), 1.0, "t>1 clamps to 1");
    }
}

#[test]
fn test_extreme_influences_and_clamping() {
    let min_inf = EaseCurve::new(0.0001, 1.0, 0.0001, 1.0).unwrap();
    assert_eq!(min_inf.out_influence, MIN_INFLUENCE);
    assert_eq!(min_inf.in_influence, MIN_INFLUENCE);
    assert!((min_inf.eval(0.5) - 0.5).abs() < 0.05);

    let max_inf = EaseCurve::new(500.0, 0.0, 500.0, 0.0).unwrap();
    assert_eq!(max_inf.out_influence, MAX_INFLUENCE);
    assert_eq!(max_inf.in_influence, MAX_INFLUENCE);
    assert_eq!(max_inf.eval(0.0), 0.0);
    assert_eq!(max_inf.eval(1.0), 1.0);
}

#[test]
fn test_linear_curve_is_identity() {
    let lin = EaseCurve::linear();
    for i in 0..=100 {
        let t = i as f64 / 100.0;
        let y = lin.eval(t);
        assert!((y - t).abs() < 1e-3, "Linear mismatch at t={t}: got {y}");
    }
}

#[test]
fn test_back_in_anticipation_dips_negative() {
    let back_in = EaseCurve::back_in();
    let mut dipped = false;
    for i in 1..50 {
        let t = i as f64 / 100.0;
        let y = back_in.eval(t);
        if y < -0.01 {
            dipped = true;
            break;
        }
    }
    assert!(dipped, "Back In must anticipate below 0");
    assert_eq!(back_in.eval(1.0), 1.0);
}

#[test]
fn test_back_out_overshoot_exceeds_one() {
    let back_out = EaseCurve::back_out();
    let mut overshot = false;
    for i in 50..99 {
        let t = i as f64 / 100.0;
        let y = back_out.eval(t);
        if y > 1.01 {
            overshot = true;
            break;
        }
    }
    assert!(overshot, "Back Out must overshoot above 1");
    assert_eq!(back_out.eval(1.0), 1.0);
}

#[test]
fn test_mirror_invariance() {
    let ease_in = EaseCurve::ease_in();
    let ease_out = ease_in.mirror();

    for i in 0..=20 {
        let t = i as f64 / 20.0;
        let y_in = ease_in.eval(t);
        let y_out = ease_out.eval(1.0 - t);
        assert!((y_in - (1.0 - y_out)).abs() < 1e-3, "Mirror mismatch at t={t}");
    }
}

#[test]
fn test_open_preset_json_roundtrip() {
    let pack = PresetPack::default();
    let json = pack.to_json().unwrap();
    let restored = PresetPack::from_json(&json).unwrap();
    assert_eq!(pack, restored);
    assert!(restored.presets.iter().any(|p| p.name == "Elastic Snap"));
    assert!(restored.presets.iter().any(|p| p.name == "Bounce Settle"));
}

#[test]
fn test_spatial_3d_path_length() {
    let a = [0.0, 0.0, 0.0];
    let b = [100.0, 0.0, 0.0];
    let straight_len = path_length_3d(a, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], b);
    assert!((straight_len - 100.0).abs() < 1e-4, "Straight line length: {straight_len}");

    let curved_len = path_length_3d(a, [0.0, 50.0, 0.0], [0.0, 50.0, 0.0], b);
    assert!(curved_len > 100.0, "Curved path must be longer than straight line: {curved_len}");
}
