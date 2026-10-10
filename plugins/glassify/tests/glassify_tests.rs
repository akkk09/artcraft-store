//! Comprehensive test suite for Glassify plug-in.

use glassify::{
    MANIFEST, default_params, ec_alloc, ec_api_version, ec_manifest_len, ec_manifest_ptr,
    render_frame,
};
use glassify::presets::{PresetConfig, PresetId};
use serde_json::Value;

#[test]
fn test_manifest_valid_json_and_schema() {
    let parsed: Value = serde_json::from_str(MANIFEST).expect("MANIFEST must be valid JSON");
    assert_eq!(parsed["api"], 1);
    assert_eq!(parsed["id"], "org.effectcraft.plugins.glassify");
    assert_eq!(parsed["name"], "Glassify");
    assert_eq!(parsed["category"], "Distort");

    let params = parsed["params"].as_array().expect("params must be an array");
    assert!(params.len() >= 20, "Should have comprehensive parameter set");

    // Check key parameters exist
    let param_ids: Vec<&str> = params.iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert!(param_ids.contains(&"preset"));
    assert!(param_ids.contains(&"refraction"));
    assert!(param_ids.contains(&"ior"));
    assert!(param_ids.contains(&"thickness"));
    assert!(param_ids.contains(&"shape_profile"));
    assert!(param_ids.contains(&"roughness"));
    assert!(param_ids.contains(&"fresnel_reflection"));
    assert!(param_ids.contains(&"chromatic_dispersion"));
    assert!(param_ids.contains(&"distortion_type"));
    assert!(param_ids.contains(&"composite_mode"));
}

#[test]
fn test_abi_exports() {
    assert_eq!(ec_api_version(), 1);

    let ptr = ec_manifest_ptr();
    let len = ec_manifest_len();
    assert!(ptr != 0);
    assert!(len > 0);

    let alloc_ptr = ec_alloc(1024);
    assert!(alloc_ptr != 0);
    assert_eq!(alloc_ptr % 8, 0, "Buffer pointer must be 8-byte aligned");
}

#[test]
fn test_render_text_like_mask() {
    let width = 128;
    let height = 64;
    let mut pixels = vec![[0.0f32; 4]; width * height];

    // Synthetic text glyph mask (bars of letters)
    for y in 16..48 {
        for x in 20..108 {
            // Glyphs: vertical bars
            if (x >= 24 && x <= 36) || (x >= 50 && x <= 62) || (x >= 76 && x <= 88) {
                pixels[y * width + x] = [1.0, 1.0, 1.0, 1.0];
            }
        }
    }

    let params = default_params();
    let res = render_frame(&mut pixels, width, height, &params, 0.0, 1.0);
    assert!(res.is_ok());

    // Verify all pixels are valid finite numbers
    for (i, px) in pixels.iter().enumerate() {
        for (c, &val) in px.iter().enumerate() {
            assert!(val.is_finite(), "Pixel {} channel {} is not finite: {}", i, c, val);
            assert!(val >= 0.0, "Color should be non-negative");
        }
    }
}

#[test]
fn test_render_transparent_logo_cutout_mode() {
    let width = 100;
    let height = 100;
    let mut pixels = vec![[0.0f32; 4]; width * height];
    let cx = 50.0f32;
    let cy = 50.0f32;

    // Circular logo on transparent background
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if (dx * dx + dy * dy).sqrt() < 30.0 {
                pixels[y * width + x] = [0.1, 0.5, 0.9, 1.0];
            }
        }
    }

    let mut params = default_params();
    // composite_mode = CutoutGlass (1)
    params[27] = 1.0;

    let res = render_frame(&mut pixels, width, height, &params, 0.0, 1.0);
    assert!(res.is_ok());

    // Corner pixel should remain completely transparent
    let corner = pixels[0];
    assert_eq!(corner[3], 0.0, "Background outside logo cutout should have alpha 0");

    // Inside logo should have non-zero glass color and alpha
    let center = pixels[50 * width + 50];
    assert!(center[3] > 0.0, "Inside glass logo should have positive alpha");
}

#[test]
fn test_render_photograph_luminance_mode() {
    let width = 80;
    let height = 80;
    let mut pixels = vec![[0.0f32; 4]; width * height];

    // Synthetic continuous photograph gradient with texture
    for y in 0..height {
        for x in 0..width {
            let lum = (x as f32 / width as f32 * 0.7 + y as f32 / height as f32 * 0.3).clamp(0.0, 1.0);
            pixels[y * width + x] = [lum, lum * 0.8, lum * 0.5, 1.0];
        }
    }

    let mut params = default_params();
    // height_source = Luminance (1)
    params[26] = 1.0;

    let res = render_frame(&mut pixels, width, height, &params, 0.0, 1.0);
    assert!(res.is_ok());

    for px in &pixels {
        assert!(px[0].is_finite() && px[1].is_finite() && px[2].is_finite() && px[3].is_finite());
    }
}

#[test]
fn test_animated_distortion_liquid_waves() {
    let width = 64;
    let height = 64;

    let mut frame_t0 = vec![[0.5f32, 0.5, 0.5, 1.0]; width * height];
    let mut frame_t1 = vec![[0.5f32, 0.5, 0.5, 1.0]; width * height];

    let mut params = default_params();
    // distortion_type = LiquidWaves (1)
    params[22] = 1.0;
    // distortion_amount = 50.0
    params[24] = 50.0;
    // distortion_speed = 2.0
    params[25] = 2.0;

    render_frame(&mut frame_t0, width, height, &params, 0.0, 1.0).unwrap();
    render_frame(&mut frame_t1, width, height, &params, 1.5, 1.0).unwrap();

    let mut diff_sum = 0.0f32;
    for i in 0..frame_t0.len() {
        diff_sum += (frame_t0[i][0] - frame_t1[i][0]).abs();
        diff_sum += (frame_t0[i][1] - frame_t1[i][1]).abs();
    }

    assert!(diff_sum > 0.1, "Animated liquid distortion must change frame across time");
}

#[test]
fn test_all_presets() {
    let width = 64;
    let height = 64;

    for preset_idx in 1..=6 {
        let preset_id = PresetId::from_index(preset_idx);
        let cfg = PresetConfig::for_preset(preset_id);
        assert!(cfg.is_some(), "Preset {} must have configuration", preset_idx);

        let mut pixels = vec![[0.3f32, 0.6, 0.9, 1.0]; width * height];
        let mut params = default_params();
        params[0] = preset_idx as f64;

        let res = render_frame(&mut pixels, width, height, &params, 0.5, 1.0);
        assert!(res.is_ok(), "Preset {} failed to render", preset_idx);

        for px in &pixels {
            assert!(px[0].is_finite() && px[1].is_finite() && px[2].is_finite() && px[3].is_finite());
        }
    }
}

#[test]
fn test_chromatic_dispersion_effect() {
    let width = 64;
    let height = 64;

    // Image with high contrast edge
    let mut base = vec![[0.0f32; 4]; width * height];
    for y in 0..height {
        for x in 0..width {
            if x < 32 {
                base[y * width + x] = [1.0, 1.0, 1.0, 1.0];
            } else {
                base[y * width + x] = [0.0, 0.0, 0.0, 1.0];
            }
        }
    }

    let mut px_no_disp = base.clone();
    let mut px_disp = base.clone();

    let mut params_no_disp = default_params();
    params_no_disp[21] = 0.0; // Chromatic Dispersion = 0

    let mut params_disp = default_params();
    params_disp[21] = 80.0; // Chromatic Dispersion = 80

    render_frame(&mut px_no_disp, width, height, &params_no_disp, 0.0, 1.0).unwrap();
    render_frame(&mut px_disp, width, height, &params_disp, 0.0, 1.0).unwrap();

    // With dispersion, R and B channels should diverge near edges
    let mut color_split = 0.0f32;
    for i in 0..px_disp.len() {
        let diff = (px_disp[i][0] - px_disp[i][2]).abs();
        color_split += diff;
    }

    assert!(color_split > 0.01, "Chromatic dispersion must create color channel difference");
}

#[test]
fn test_resolution_scale_stability() {
    let mut params = default_params();
    // Test Lucid preset
    params[0] = 1.0;

    for &scale in &[1.0, 0.5, 0.25] {
        let w = (64.0 * scale) as usize;
        let h = (64.0 * scale) as usize;
        let mut pixels = vec![[0.5f32, 0.5, 0.5, 1.0]; w * h];

        let res = render_frame(&mut pixels, w, h, &params, 0.0, scale);
        assert!(res.is_ok(), "Failed at scale {}", scale);

        for px in &pixels {
            assert!(px[0].is_finite() && px[1].is_finite() && px[2].is_finite() && px[3].is_finite());
        }
    }
}

#[test]
fn test_quality_settings() {
    let width = 32;
    let height = 32;

    for q in 0..=2 {
        let mut pixels = vec![[0.5f32, 0.5, 0.5, 1.0]; width * height];
        let mut params = default_params();
        params[6] = 50.0; // roughness
        params[28] = q as f64; // quality: Draft, Good, Cinematic

        let res = render_frame(&mut pixels, width, height, &params, 0.0, 1.0);
        assert!(res.is_ok(), "Quality setting {} failed", q);
    }
}

#[test]
fn test_render_frame_invalid_dimensions() {
    let mut pixels = vec![[0.5f32, 0.5, 0.5, 1.0]; 32 * 32];
    let params = default_params();

    let res_zero = render_frame(&mut pixels, 0, 0, &params, 0.0, 1.0);
    assert!(res_zero.is_err(), "render_frame should fail on 0x0 dimensions");

    let res_short = render_frame(&mut pixels[0..10], 32, 32, &params, 0.0, 1.0);
    assert!(res_short.is_err(), "render_frame should fail when slice is too short");
}
