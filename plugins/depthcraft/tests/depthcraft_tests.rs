use depthcraft::filter::{blend_fog, depth_slice_highlight, refine_depth_edges, remap_depth, turbo_colormap};
use depthcraft::models::{MODELS, find_model, verify_weights};
use depthcraft::pipeline::{CancellationToken, DepthPipeline, PipelineConfig};
use depthcraft::temporal::{apply_temporal_smoothing, reset_temporal_state};
use depthcraft::{MANIFEST, ec_alloc, ec_api_version, ec_manifest_len, ec_manifest_ptr, render_frame};

#[test]
fn test_plugin_abi_exports() {
    assert_eq!(ec_api_version(), 1);

    let ptr = ec_manifest_ptr();
    let len = ec_manifest_len();
    assert!(ptr != 0);
    assert_eq!(len as usize, MANIFEST.len());

    let alloc_ptr = ec_alloc(1024);
    assert!(alloc_ptr != 0);
    assert_eq!(alloc_ptr % 8, 0, "ec_alloc must return 8-byte aligned buffer");
}

#[test]
fn test_manifest_is_valid_json() {
    let parsed: serde_json::Value = serde_json::from_str(MANIFEST).expect("manifest must be valid JSON");
    assert_eq!(parsed["api"], 1);
    assert_eq!(parsed["id"], "org.effectcraft.plugins.depthcraft");
    assert_eq!(parsed["name"], "DepthCraft");
    assert_eq!(parsed["category"], "Stylize");

    let params = parsed["params"].as_array().expect("params must be array");
    assert_eq!(params.len(), 11);

    let param_ids: Vec<&str> = params.iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert!(param_ids.contains(&"mode"));
    assert!(param_ids.contains(&"near_depth"));
    assert!(param_ids.contains(&"far_depth"));
    assert!(param_ids.contains(&"invert"));
    assert!(param_ids.contains(&"gamma"));
    assert!(param_ids.contains(&"temporal_smooth"));
    assert!(param_ids.contains(&"edge_refine"));
    assert!(param_ids.contains(&"fog_density"));
    assert!(param_ids.contains(&"fog_color"));
    assert!(param_ids.contains(&"slice_center"));
    assert!(param_ids.contains(&"slice_width"));
}

#[test]
fn test_remap_depth_and_inversion() {
    // Normal 0..100 range
    assert!((remap_depth(0.0, 0.0, 100.0, false, 1.0) - 0.0).abs() < 1e-4);
    assert!((remap_depth(0.5, 0.0, 100.0, false, 1.0) - 0.5).abs() < 1e-4);
    assert!((remap_depth(1.0, 0.0, 100.0, false, 1.0) - 1.0).abs() < 1e-4);

    // Near/Far range [20, 80]
    assert!((remap_depth(0.1, 20.0, 80.0, false, 1.0) - 0.0).abs() < 1e-4); // Below near -> clamped to 0
    assert!((remap_depth(0.2, 20.0, 80.0, false, 1.0) - 0.0).abs() < 1e-4); // At near
    assert!((remap_depth(0.5, 20.0, 80.0, false, 1.0) - 0.5).abs() < 1e-4); // Midpoint
    assert!((remap_depth(0.8, 20.0, 80.0, false, 1.0) - 1.0).abs() < 1e-4); // At far
    assert!((remap_depth(0.9, 20.0, 80.0, false, 1.0) - 1.0).abs() < 1e-4); // Above far -> clamped to 1

    // Invert
    assert!((remap_depth(0.0, 0.0, 100.0, true, 1.0) - 1.0).abs() < 1e-4);
    assert!((remap_depth(1.0, 0.0, 100.0, true, 1.0) - 0.0).abs() < 1e-4);
    assert!((remap_depth(0.3, 0.0, 100.0, true, 1.0) - 0.7).abs() < 1e-4);

    // Gamma curve
    let g2 = remap_depth(0.25, 0.0, 100.0, false, 2.0);
    assert!((g2 - 0.5).abs() < 1e-4); // sqrt(0.25) = 0.5
}

#[test]
fn test_bilateral_edge_refinement() {
    let width = 10;
    let height = 10;
    let mut rgb = vec![[0.0f32; 3]; width * height];
    let mut depth = vec![0.0f32; width * height];

    // Left half is black, right half is bright white
    // Depth has a noisy step near the boundary
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if x >= 5 {
                rgb[idx] = [1.0, 1.0, 1.0];
                depth[idx] = 0.9;
            } else {
                rgb[idx] = [0.0, 0.0, 0.0];
                depth[idx] = 0.1;
            }
        }
    }

    // Add noise pixel on the boundary
    depth[5 * width + 4] = 0.7; // False high depth on the dark side

    let mut refined = vec![0.0f32; width * height];
    refine_depth_edges(&depth, &rgb, width, height, 2, &mut refined);

    // After refinement, the dark side pixel should be pulled down towards its dark neighbours
    assert!(refined[5 * width + 4] < 0.5, "noisy boundary pixel should be smoothed towards luminance domain");
    assert!(refined[5 * width + 7] > 0.8, "solid side should preserve high depth");
    assert!(refined[5 * width + 1] < 0.2, "solid side should preserve low depth");
}

#[test]
fn test_temporal_smoothing_reduces_flicker() {
    reset_temporal_state();

    let width = 4;
    let height = 4;
    let lum = vec![0.5f32; width * height];

    // Frame 1
    let mut d1 = vec![0.5f32; width * height];
    apply_temporal_smoothing(&mut d1, &lum, width, height, 0.0, 80.0);
    assert_eq!(d1[0], 0.5);

    // Frame 2 with slight high-frequency noise (+0.2)
    let mut d2 = vec![0.7f32; width * height];
    apply_temporal_smoothing(&mut d2, &lum, width, height, 0.04, 80.0);

    // Smoothed frame 2 should be dampened towards 0.5
    assert!(d2[0] < 0.65, "flicker should be dampened: got {}", d2[0]);
    assert!(d2[0] > 0.50, "flicker should still track new frame: got {}", d2[0]);

    // Test scene cut (huge luminance jump)
    let cut_lum = vec![0.95f32; width * height];
    let mut d_cut = vec![0.1f32; width * height];
    apply_temporal_smoothing(&mut d_cut, &cut_lum, width, height, 0.08, 80.0);

    // Edge gating should allow cut through without trailing ghost
    assert!(d_cut[0] < 0.35, "scene cut should adapt quickly without ghosting: got {}", d_cut[0]);
}

#[test]
fn test_turbo_colormap_and_modes() {
    for i in 0..=10 {
        let t = i as f32 / 10.0;
        let c = turbo_colormap(t);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch), "colormap output must be in [0, 1]");
        }
    }

    // Fog blending
    let src = [0.2, 0.4, 0.8];
    let fog_color = [0.9, 0.9, 0.9];
    let fogged_near = blend_fog(src, 0.0, 50.0, fog_color);
    assert!((fogged_near[0] - src[0]).abs() < 1e-4, "near depth should have no fog");

    let fogged_far = blend_fog(src, 1.0, 100.0, fog_color);
    assert!((fogged_far[0] - fog_color[0]).abs() < 1e-4, "full far depth should have 100% fog");

    // Depth slice
    let hl_center = depth_slice_highlight(0.5, 50.0, 20.0);
    assert_eq!(hl_center, 1.0, "center of slice should be maximum highlight");
    let hl_outside = depth_slice_highlight(0.8, 50.0, 20.0);
    assert_eq!(hl_outside, 0.0, "outside slice should be zero highlight");
}

#[test]
fn test_models_registry_and_verification() {
    assert!(MODELS.len() >= 3);
    for m in MODELS {
        assert!(!m.id.is_empty());
        assert!(!m.name.is_empty());
        assert!(!m.authors.is_empty());
        assert!(!m.license.is_empty());
        assert!(m.license_url.starts_with("https://"));
        assert!(m.homepage.starts_with("https://"));
    }

    let classical = find_model("classical").expect("classical model must exist");
    assert!(verify_weights(classical, b"any bytes").is_ok());

    let midas = find_model("midas_v21_small").expect("midas model must exist");
    assert_eq!(midas.license, "MIT");
    assert!(verify_weights(midas, b"wrong bytes").is_err());
}

#[test]
fn test_pipeline_caching_and_cancellation() {
    let config = PipelineConfig {
        model_id: "classical".to_string(),
        quality: depthcraft::models::QualityPreset::Fast,
        edge_refine_radius: 1,
        temporal_smooth_weight: 0.0,
        memory_limit_bytes: 1024 * 1024,
    };
    let mut pipeline = DepthPipeline::new(config);

    let frame = vec![[0.5f32; 3]; 16 * 16];
    let (d1, hit1) = pipeline.process_frame(&frame, 16, 16);
    assert!(!hit1, "first call should be a cache miss");
    assert_eq!(d1.len(), 256);

    let (d2, hit2) = pipeline.process_frame(&frame, 16, 16);
    assert!(hit2, "second call with identical frame should be a cache hit");
    assert_eq!(d1, d2);

    // Test cancellation
    let cancel = CancellationToken::new();
    cancel.cancel();

    let seq_frames = [(&frame[..], 16, 16)];
    let res = pipeline.process_sequence(&seq_frames, None, &cancel);
    assert!(res.is_err(), "pipeline should abort when cancellation token is set");
}

#[test]
fn test_render_frame_execution() {
    let width = 8;
    let height = 8;
    let nparams = 14;

    let mut params = vec![0.0f64; nparams];
    params[0] = 0.0; // Mode: Grayscale Depth
    params[1] = 0.0; // Near: 0
    params[2] = 100.0; // Far: 100
    params[3] = 0.0; // Invert: false
    params[4] = 1.0; // Gamma: 1.0
    params[5] = 0.0; // Temporal smooth: 0
    params[6] = 1.0; // Edge refine: 1
    params[7] = 50.0; // Fog density: 50
    params[8] = 0.8; // Fog color R
    params[9] = 0.85; // Fog color G
    params[10] = 0.9; // Fog color B
    params[11] = 1.0; // Fog color A
    params[12] = 50.0; // Slice center
    params[13] = 20.0; // Slice width

    let mut pixels = vec![[0.0f32; 4]; width * height];
    for (i, px) in pixels.iter_mut().enumerate() {
        let y = (i / width) as f32 / height as f32;
        *px = [0.8 * y, 0.4 * y, 0.2 * y, 1.0];
    }

    let ret = render_frame(&mut pixels, width, height, &params, 0.0);
    assert!(ret.is_ok(), "render_frame should succeed");

    // Pixels should now be grayscale depth map
    for px in pixels.iter() {
        assert_eq!(px[0], px[1], "grayscale mode must have R == G");
        assert_eq!(px[1], px[2], "grayscale mode must have G == B");
        assert_eq!(px[3], 1.0, "alpha should be preserved");
    }
}
