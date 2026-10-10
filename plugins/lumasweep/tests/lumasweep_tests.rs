//! Comprehensive unit and integration test suite for LumaSweep.

use lumasweep::texture::sample_surface_texture;
use lumasweep::{MANIFEST, default_params, ec_alloc, ec_api_version, ec_manifest_len, ec_manifest_ptr, render_frame};

#[test]
fn test_plugin_abi_exports() {
    assert_eq!(ec_api_version(), 1);

    let ptr = ec_manifest_ptr();
    let len = ec_manifest_len();
    assert!(ptr != 0);
    assert_eq!(len as usize, MANIFEST.len());

    let alloc_ptr = ec_alloc(2048);
    assert!(alloc_ptr != 0);
    assert_eq!(alloc_ptr % 8, 0, "ec_alloc must be 8-byte aligned");
}

#[test]
fn test_manifest_is_valid_json() {
    let parsed: serde_json::Value = serde_json::from_str(MANIFEST).expect("manifest must be valid JSON");
    assert_eq!(parsed["api"], 1);
    assert_eq!(parsed["id"], "org.effectcraft.plugins.lumasweep");
    assert_eq!(parsed["name"], "LumaSweep");
    assert_eq!(parsed["category"], "Generate");

    let params = parsed["params"].as_array().expect("params must be array");
    assert_eq!(params.len(), 25);

    let ids: Vec<&str> = params.iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert!(ids.contains(&"preset"));
    assert!(ids.contains(&"center"));
    assert!(ids.contains(&"direction"));
    assert!(ids.contains(&"width"));
    assert!(ids.contains(&"sweep_intensity"));
    assert!(ids.contains(&"shape"));
    assert!(ids.contains(&"softness"));
    assert!(ids.contains(&"highlight_color"));
    assert!(ids.contains(&"shadow_color"));
    assert!(ids.contains(&"shadow_intensity"));
    assert!(ids.contains(&"bevel_intensity"));
    assert!(ids.contains(&"bevel_depth"));
    assert!(ids.contains(&"bevel_profile"));
    assert!(ids.contains(&"reception_mode"));
    assert!(ids.contains(&"edge_source"));
    assert!(ids.contains(&"glow_intensity"));
    assert!(ids.contains(&"glow_radius"));
    assert!(ids.contains(&"chromatic_fringe"));
    assert!(ids.contains(&"texture_intensity"));
    assert!(ids.contains(&"texture_scale"));
    assert!(ids.contains(&"texture_mode"));
    assert!(ids.contains(&"auto_animate"));
    assert!(ids.contains(&"anim_speed"));
    assert!(ids.contains(&"anim_loop"));
    assert!(ids.contains(&"anim_phase"));
}

#[test]
fn test_text_like_mask_rendering() {
    let width = 64;
    let height = 32;
    let mut pixels = vec![[0.0f32; 4]; width * height];

    // Create a text-like horizontal bar in the center
    for y in 0..height {
        for x in 0..width {
            let in_bar = y >= 10 && y <= 22 && x >= 12 && x <= 52;
            let a = if in_bar { 1.0f32 } else { 0.0f32 };
            pixels[y * width + x] = [0.5 * a, 0.5 * a, 0.5 * a, a];
        }
    }

    let mut params = default_params();
    params[0] = 0.0; // Custom
    params[1] = 32.0; // Center x
    params[2] = 16.0; // Center y
    params[3] = 0.0; // Direction: vertical sweep line
    params[4] = 20.0; // Width
    params[5] = 100.0; // Sweep intensity
    params[17] = 150.0; // Bevel intensity
    params[18] = 4.0; // Bevel depth

    let mut rendered = pixels.clone();
    render_frame(&mut rendered, width, height, &params, 0.0, 1.0).expect("render failed");

    // Pixels outside the text bar must remain completely transparent (0.0 alpha)
    assert_eq!(rendered[0][3], 0.0, "outside text mask must remain transparent");
    assert_eq!(rendered[5 * width + 5][3], 0.0);

    // Center pixel of the text bar should be illuminated
    let center_idx = 16 * width + 32;
    assert!(rendered[center_idx][0] > pixels[center_idx][0], "center of text must receive light sweep highlight");
    assert_eq!(rendered[center_idx][3], 1.0, "alpha is preserved");

    // Edge pixels (e.g. at x = 12, y = 16) should exhibit bevel edge enhancement
    let edge_idx = 16 * width + 13;
    assert!(rendered[edge_idx][0] > 0.5, "bevel edge should catch specular highlight");
}

#[test]
fn test_transparent_image_logo() {
    let width = 50;
    let height = 50;
    let mut pixels = vec![[0.0f32; 4]; width * height];

    // Circular logo in center
    for y in 0..height {
        let dy = y as f32 - 25.0;
        for x in 0..width {
            let dx = x as f32 - 25.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let a = if dist <= 18.0 { 1.0f32 } else { 0.0f32 };
            pixels[y * width + x] = [0.2 * a, 0.4 * a, 0.8 * a, a];
        }
    }

    let mut params = default_params();
    params[0] = 0.0;
    params[1] = 25.0;
    params[2] = 25.0;
    params[3] = -45.0;
    params[4] = 30.0;
    params[5] = 120.0;

    let mut rendered = pixels.clone();
    render_frame(&mut rendered, width, height, &params, 0.0, 1.0).expect("render failed");

    let logo_center = 25 * width + 25;
    assert!(rendered[logo_center][0] > pixels[logo_center][0], "logo center receives highlight");
    assert_eq!(rendered[0][3], 0.0, "corner transparent background preserved");
}

#[test]
fn test_flat_image_luminance_driven_relief() {
    let width = 40;
    let height = 40;
    let mut pixels = vec![[0.0f32; 4]; width * height];

    // Fully opaque image (alpha = 1.0 everywhere) with a high-contrast bright square in the middle
    for y in 0..height {
        for x in 0..width {
            let in_box = x >= 15 && x <= 25 && y >= 15 && y <= 25;
            let val = if in_box { 0.9f32 } else { 0.1f32 };
            pixels[y * width + x] = [val, val, val, 1.0];
        }
    }

    let mut params = default_params();
    params[0] = 0.0;
    params[1] = 20.0;
    params[2] = 20.0;
    params[4] = 25.0;
    params[5] = 100.0;
    params[17] = 200.0; // Bevel intensity
    params[18] = 4.0; // Bevel depth
    params[21] = 1.0; // Edge source: Luminance!

    let mut rendered = pixels.clone();
    render_frame(&mut rendered, width, height, &params, 0.0, 1.0).expect("render failed");

    // All alpha must remain 1.0
    for p in &rendered {
        assert_eq!(p[3], 1.0);
    }

    // Edge of the bright square (e.g. at x = 16, y = 20) receives bevel response derived from luminance!
    let edge_idx = 20 * width + 16;
    assert!(rendered[edge_idx][0] > pixels[edge_idx][0], "luminance boundary should catch bevel highlight");
}

#[test]
fn test_animated_sweep_progresses_with_time() {
    let width = 60;
    let height = 40;
    let pixels = vec![[0.5f32, 0.5f32, 0.5f32, 1.0f32]; width * height];

    let mut params = default_params();
    params[0] = 0.0;
    params[3] = 0.0; // Sweep moving left-to-right
    params[4] = 15.0; // Width
    params[5] = 150.0;
    params[28] = 1.0; // Auto-animate enabled
    params[29] = 1.0; // Speed 1 cycle / sec

    let mut frame_t0 = pixels.clone();
    render_frame(&mut frame_t0, width, height, &params, 0.0, 1.0).expect("render frame 0 failed");

    let mut frame_t1 = pixels.clone();
    render_frame(&mut frame_t1, width, height, &params, 0.25, 1.0).expect("render frame 1 failed");

    assert_ne!(frame_t0, frame_t1, "sweep should change pixel lighting across time");
}

#[test]
fn test_resolution_aware_scale_stability() {
    let width_full = 80;
    let height_full = 80;
    let px_full = vec![[0.4f32, 0.4f32, 0.4f32, 1.0f32]; width_full * height_full];

    let mut params = default_params();
    params[0] = 0.0;
    params[1] = 40.0;
    params[2] = 40.0;
    params[4] = 30.0;
    params[5] = 100.0;

    let mut rend_full = px_full.clone();
    render_frame(&mut rend_full, width_full, height_full, &params, 0.0, 1.0).expect("full render failed");

    // Half scale preview
    let width_half = 40;
    let height_half = 40;
    let px_half = vec![[0.4f32, 0.4f32, 0.4f32, 1.0f32]; width_half * height_half];

    let mut params_half = params.clone();
    params_half[1] = 20.0;
    params_half[2] = 20.0;

    let mut rend_half = px_half.clone();
    render_frame(&mut rend_half, width_half, height_half, &params_half, 0.0, 0.5).expect("half render failed");

    // The center pixel brightness in both should be closely matched
    let center_full = 40 * width_full + 40;
    let center_half = 20 * width_half + 20;
    let diff = (rend_full[center_full][0] - rend_half[center_half][0]).abs();
    assert!(diff < 0.08, "peak brightness across resolutions must remain stable (diff={diff})");
}

#[test]
fn test_presets_execution() {
    let width = 40;
    let height = 40;
    let base_pixels = vec![[0.5f32, 0.5f32, 0.5f32, 1.0f32]; width * height];

    for preset_idx in 1..=6 {
        let mut pixels = base_pixels.clone();
        let mut params = default_params();
        params[0] = preset_idx as f64;
        params[1] = 20.0;
        params[2] = 20.0;

        render_frame(&mut pixels, width, height, &params, 0.0, 1.0)
            .unwrap_or_else(|_| panic!("preset {} failed to render", preset_idx));

        let center_idx = 20 * width + 20;
        assert!(pixels[center_idx][0] > 0.0);
    }
}

#[test]
fn test_prism_chromatic_fringe_dispersion() {
    let width = 60;
    let height = 60;
    let mut pixels = vec![[0.2f32, 0.2f32, 0.2f32, 1.0f32]; width * height];

    let mut params = default_params();
    params[0] = 3.0; // Prism preset
    params[1] = 30.0;
    params[2] = 30.0;
    params[3] = 0.0;

    render_frame(&mut pixels, width, height, &params, 0.0, 1.0).expect("prism render failed");

    // Slightly off-center pixel across the wavefront should have chromatic color difference (R != B)
    let off_center = 30 * width + 33;
    let r = pixels[off_center][0];
    let b = pixels[off_center][2];
    assert!((r - b).abs() > 0.01, "prism preset should produce spectral chromatic fringe (R={r}, B={b})");
}

#[test]
fn test_procedural_texture_determinism() {
    let sample1 = sample_surface_texture(12.5, 45.2, 0, 25.0);
    let sample2 = sample_surface_texture(12.5, 45.2, 0, 25.0);
    assert_eq!(sample1, sample2, "procedural texture must be strictly deterministic");

    let sample_other = sample_surface_texture(14.0, 45.2, 0, 25.0);
    assert_ne!(sample1, sample_other, "different coordinates should produce varied grain");
}
