//! LumaSweep: Advanced controllable light sweep effect plug-in for EffectCraft.
//!
//! Complies with EffectCraft Effect Plug-in API v1 (see `docs/plugins.md`).
//! Produces polished highlights on text, logos, vector shapes, and raster images with
//! bevel edge response, chromatic fringe dispersion, procedural micro-surface texture,
//! time-based animation, and resolution-aware rendering.

pub mod bevel;
pub mod presets;
pub mod sweep;
pub mod texture;

use std::cell::RefCell;

use bevel::{BevelConfig, SurfaceField};
use presets::{PresetConfig, PresetId};
use sweep::{BeamProfile, compute_center};
use texture::sample_surface_texture;

pub const MANIFEST: &str = r#"{
  "api": 1,
  "id": "org.effectcraft.plugins.lumasweep",
  "name": "LumaSweep",
  "category": "Generate",
  "version": "1.0.0",
  "author": "EffectCraft contributors",
  "description": "Controllable light sweep with bevel edge response, chromatic fringe, procedural micro-texture, and luminance relief.",
  "params": [
    {"id": "preset", "name": "Preset", "type": "popup", "options": ["Custom", "Chrome", "Soft Studio", "Prism", "Brushed Metal", "Gold Lustre", "Laser Beam"], "default": 0},
    {"id": "center", "name": "Center", "type": "point", "default": [0.5, 0.5]},
    {"id": "direction", "name": "Direction", "type": "angle", "default": -35.0},
    {"id": "width", "name": "Width", "type": "slider", "default": 80.0, "min": 2.0, "max": 2000.0, "sliderMax": 500.0, "decimals": 1},
    {"id": "sweep_intensity", "name": "Sweep Intensity", "type": "slider", "default": 100.0, "min": 0.0, "max": 500.0, "sliderMax": 200.0, "decimals": 1},
    {"id": "shape", "name": "Sweep Shape", "type": "popup", "options": ["Smooth Hermite", "Linear", "Sharp Peak", "Gaussian", "Asymmetric"], "default": 0},
    {"id": "softness", "name": "Softness", "type": "slider", "default": 50.0, "min": 0.0, "max": 100.0, "decimals": 1},
    {"id": "highlight_color", "name": "Highlight Color", "type": "color", "default": [1.0, 1.0, 1.0, 1.0]},
    {"id": "shadow_color", "name": "Shadow Color", "type": "color", "default": [0.05, 0.05, 0.1, 1.0]},
    {"id": "shadow_intensity", "name": "Shadow Intensity", "type": "slider", "default": 25.0, "min": 0.0, "max": 100.0, "decimals": 1},
    {"id": "bevel_intensity", "name": "Bevel Intensity", "type": "slider", "default": 120.0, "min": 0.0, "max": 500.0, "sliderMax": 250.0, "decimals": 1},
    {"id": "bevel_depth", "name": "Bevel Depth", "type": "slider", "default": 4.0, "min": 0.0, "max": 50.0, "sliderMax": 20.0, "decimals": 1},
    {"id": "bevel_profile", "name": "Bevel Profile", "type": "popup", "options": ["Curved (Glossy)", "Chisel (Sharp)", "Ridge (Stepped)", "Rim (Accent)"], "default": 0},
    {"id": "reception_mode", "name": "Light Reception", "type": "popup", "options": ["Add", "Composite / Blend", "Cutout (Light Only)", "Screen"], "default": 0},
    {"id": "edge_source", "name": "Edge / Relief Source", "type": "popup", "options": ["Alpha (Logos & Text)", "Luminance (Photos & Artwork)", "Combined (Alpha + Luma)"], "default": 0},
    {"id": "glow_intensity", "name": "Glow Intensity", "type": "slider", "default": 0.0, "min": 0.0, "max": 300.0, "sliderMax": 150.0, "decimals": 1},
    {"id": "glow_radius", "name": "Glow Radius", "type": "slider", "default": 30.0, "min": 1.0, "max": 200.0, "sliderMax": 100.0, "decimals": 1},
    {"id": "chromatic_fringe", "name": "Chromatic Fringe", "type": "slider", "default": 0.0, "min": 0.0, "max": 100.0, "decimals": 1},
    {"id": "texture_intensity", "name": "Texture Intensity", "type": "slider", "default": 0.0, "min": 0.0, "max": 100.0, "decimals": 1},
    {"id": "texture_scale", "name": "Texture Scale", "type": "slider", "default": 25.0, "min": 1.0, "max": 200.0, "sliderMax": 100.0, "decimals": 1},
    {"id": "texture_mode", "name": "Texture Mode", "type": "popup", "options": ["Brushed Anisotropic", "Micro Grain", "Satin Cross"], "default": 0},
    {"id": "auto_animate", "name": "Auto Animate", "type": "checkbox", "default": false},
    {"id": "anim_speed", "name": "Sweep Speed", "type": "slider", "default": 0.5, "min": -10.0, "max": 10.0, "sliderMin": -5.0, "sliderMax": 5.0, "decimals": 2},
    {"id": "anim_loop", "name": "Loop Mode", "type": "popup", "options": ["One-Way (Repeat)", "Ping-Pong (Bounce)"], "default": 0},
    {"id": "anim_phase", "name": "Sweep Phase", "type": "slider", "default": 0.0, "min": 0.0, "max": 360.0, "decimals": 1}
  ]
}"#;

thread_local! {
    static BUF: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn ec_api_version() -> i32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn ec_manifest_ptr() -> i32 {
    MANIFEST.as_ptr() as usize as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn ec_manifest_len() -> i32 {
    MANIFEST.len() as i32
}

/// One reusable, 8-byte-aligned buffer for the parameters and pixels.
#[unsafe(no_mangle)]
pub extern "C" fn ec_alloc(bytes: i32) -> i32 {
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        let words = (bytes.max(0) as usize).div_ceil(8);
        if b.len() < words {
            b.resize(words, 0);
        }
        b.as_mut_ptr() as usize as i32
    })
}

pub fn default_params() -> Vec<f64> {
    vec![
        0.0,                 // 0: preset (Custom)
        0.5, 0.5,            // 1, 2: center
        -35.0,               // 3: direction
        80.0,                // 4: width
        100.0,               // 5: sweep_intensity
        0.0,                 // 6: shape (Smooth Hermite)
        50.0,                // 7: softness
        1.0, 1.0, 1.0, 1.0,  // 8..11: highlight_color (white)
        0.05, 0.05, 0.1, 1.0,// 12..15: shadow_color
        25.0,                // 16: shadow_intensity
        120.0,               // 17: bevel_intensity
        4.0,                 // 18: bevel_depth
        0.0,                 // 19: bevel_profile (Curved)
        0.0,                 // 20: reception_mode (Add)
        0.0,                 // 21: edge_source (Alpha)
        0.0,                 // 22: glow_intensity
        30.0,                // 23: glow_radius
        0.0,                 // 24: chromatic_fringe
        0.0,                 // 25: texture_intensity
        25.0,                // 26: texture_scale
        0.0,                 // 27: texture_mode
        0.0,                 // 28: auto_animate (false)
        0.5,                 // 29: anim_speed
        0.0,                 // 30: anim_loop
        0.0,                 // 31: anim_phase
    ]
}

/// Core rendering logic for LumaSweep, processing pixels in-place.
pub fn render_frame(
    px: &mut [[f32; 4]],
    width: usize,
    height: usize,
    params: &[f64],
    time: f64,
    scale: f64,
) -> Result<(), &'static str> {
    if width == 0 || height == 0 || px.len() < width * height {
        return Err("invalid frame dimensions");
    }

    let resolution_scale = if scale > 0.001 { scale as f32 } else { 1.0f32 };

    // Parse parameters with defaults
    let preset_idx = params.first().copied().unwrap_or(0.0).round().max(0.0) as usize;
    let preset_id = PresetId::from_index(preset_idx);
    let preset_cfg = PresetConfig::for_preset(preset_id);

    // Center coordinates
    let raw_cx = params.get(1).copied().unwrap_or(0.5);
    let raw_cy = params.get(2).copied().unwrap_or(0.5);
    let manual_center = if raw_cx <= 1.0 && raw_cy <= 1.0 && width > 1 && height > 1 {
        (raw_cx * width as f64, raw_cy * height as f64)
    } else {
        (raw_cx, raw_cy)
    };

    let dir_deg = params.get(3).copied().unwrap_or(-35.0);
    let dir_rad = dir_deg.to_radians();

    let mut width_val = params.get(4).copied().unwrap_or(80.0) as f32;
    let mut sweep_intensity = (params.get(5).copied().unwrap_or(100.0) as f32) / 100.0;
    let mut shape = params.get(6).copied().unwrap_or(0.0).round().max(0.0) as usize;
    let mut softness = params.get(7).copied().unwrap_or(50.0) as f32;

    let raw_hl_r = params.get(8).copied().unwrap_or(1.0) as f32;
    let raw_hl_g = params.get(9).copied().unwrap_or(1.0) as f32;
    let raw_hl_b = params.get(10).copied().unwrap_or(1.0) as f32;
    let raw_hl_a = params.get(11).copied().unwrap_or(1.0) as f32;
    let mut hl_col = if raw_hl_a <= 0.0 && raw_hl_r == 0.0 && raw_hl_g == 0.0 && raw_hl_b == 0.0 {
        [1.0, 1.0, 1.0, 1.0]
    } else {
        [raw_hl_r, raw_hl_g, raw_hl_b, if raw_hl_a > 0.0 { raw_hl_a } else { 1.0 }]
    };

    let mut sh_col = [
        params.get(12).copied().unwrap_or(0.05) as f32,
        params.get(13).copied().unwrap_or(0.05) as f32,
        params.get(14).copied().unwrap_or(0.1) as f32,
        params.get(15).copied().unwrap_or(1.0) as f32,
    ];

    let mut sh_intensity = params.get(16).copied().unwrap_or(25.0) as f32;
    let mut bevel_intensity = params.get(17).copied().unwrap_or(120.0) as f32;
    let mut bevel_depth = params.get(18).copied().unwrap_or(4.0) as f32;
    let mut bevel_profile = params.get(19).copied().unwrap_or(0.0).round().max(0.0) as usize;
    let mut reception_mode = params.get(20).copied().unwrap_or(0.0).round().max(0.0) as usize;
    let edge_source = params.get(21).copied().unwrap_or(0.0).round().max(0.0) as usize;

    let mut glow_intensity = params.get(22).copied().unwrap_or(0.0) as f32;
    let mut glow_radius = params.get(23).copied().unwrap_or(30.0) as f32;
    let mut chromatic_fringe = params.get(24).copied().unwrap_or(0.0) as f32;

    let mut texture_intensity = params.get(25).copied().unwrap_or(0.0) as f32;
    let mut texture_scale = params.get(26).copied().unwrap_or(25.0) as f32;
    let mut texture_mode = params.get(27).copied().unwrap_or(0.0).round().max(0.0) as usize;

    let auto_animate = params.get(28).copied().unwrap_or(0.0) != 0.0;
    let anim_speed = params.get(29).copied().unwrap_or(0.5);
    let anim_loop = params.get(30).copied().unwrap_or(0.0).round().max(0.0) as usize;
    let anim_phase = params.get(31).copied().unwrap_or(0.0);

    // If preset is selected, apply preset parameter configuration
    if let Some(cfg) = preset_cfg {
        shape = cfg.shape;
        softness = cfg.softness;
        width_val = cfg.width;
        sweep_intensity = cfg.sweep_intensity / 100.0;
        hl_col = cfg.highlight_color;
        sh_col = cfg.shadow_color;
        sh_intensity = cfg.shadow_intensity;
        bevel_intensity = cfg.bevel_intensity;
        bevel_depth = cfg.bevel_depth;
        bevel_profile = cfg.bevel_profile;
        reception_mode = cfg.reception_mode;
        chromatic_fringe = cfg.chromatic_fringe;
        texture_intensity = cfg.texture_intensity;
        texture_scale = cfg.texture_scale;
        texture_mode = cfg.texture_mode;
        glow_intensity = cfg.glow_intensity;
        glow_radius = cfg.glow_radius;
    }

    // Resolution-aware scaling
    let width_px = (width_val * resolution_scale).max(1.0);
    let bevel_depth_px = bevel_depth * resolution_scale;
    let glow_radius_px = glow_radius * resolution_scale;
    let texture_scale_px = texture_scale * resolution_scale;

    // Time-based animation center
    let center = compute_center(
        manual_center,
        width,
        height,
        dir_rad,
        auto_animate,
        anim_speed,
        anim_loop,
        anim_phase,
        time,
    );

    // Precompute beam profile
    let beam = BeamProfile {
        shape,
        width: width_px,
        softness,
        chromatic_fringe,
        glow_intensity,
        glow_radius: glow_radius_px,
    };

    // Precompute heightfield & edge gradients
    let surface = SurfaceField::build(px, width, height, edge_source, bevel_depth_px);

    let bevel_cfg = BevelConfig {
        intensity: bevel_intensity,
        depth: bevel_depth_px,
        profile: bevel_profile,
        edge_source,
        shadow_intensity: sh_intensity,
    };

    let nx = dir_rad.cos() as f32;
    let ny = dir_rad.sin() as f32;
    let light_dir = (nx, ny);

    let cos_d = dir_rad.cos();
    let sin_d = dir_rad.sin();
    let cx = center.0;
    let cy = center.1;

    for y in 0..height {
        let row_offset = y * width;
        let dy = y as f64 + 0.5 - cy;

        for x in 0..width {
            let idx = row_offset + x;
            let c = &mut px[idx];
            let a = c[3];

            if a <= 1e-5 && reception_mode != 2 {
                continue;
            }

            let dx = x as f64 + 0.5 - cx;

            // Distance perpendicular to sweep beam
            let dist_perp = (dx * cos_d + dy * sin_d) as f32;
            // Coordinate parallel to sweep beam (along wavefront)
            let coord_par = (-dx * sin_d + dy * cos_d) as f64;

            // 1. Beam falloff with chromatic dispersion
            let b_spec = beam.evaluate_chromatic(dist_perp);

            // 2. Glow halo
            let glow = beam.evaluate_glow(dist_perp);

            // 3. Bevel edge shading and rim shadow
            let (edge_hl, rim_sh) = surface.evaluate_shading(x, y, &bevel_cfg, light_dir);

            // 4. Procedural micro-surface texture
            let tex_mod = if texture_intensity > 0.001 {
                let sample = sample_surface_texture(dist_perp as f64, coord_par, texture_mode, texture_scale_px);
                1.0 + sample * (texture_intensity / 100.0)
            } else {
                1.0
            };

            // Calculate per-channel light addition
            let mut l_rgb = [0.0f32; 3];
            for i in 0..3 {
                let beam_component = b_spec[i] * sweep_intensity * tex_mod;
                let edge_component = edge_hl * (b_spec[i].max(0.15));
                let total_light = beam_component + edge_component;
                l_rgb[i] = total_light * hl_col[i] + glow * hl_col[i];
            }

            // Un-premultiply source RGB
            let (r0, g0, b0) = if a > 1e-4 {
                (c[0] / a, c[1] / a, c[2] / a)
            } else {
                (0.0, 0.0, 0.0)
            };

            // Apply shadow/ambient occlusion from rim shadow
            let shadow_factor = (1.0 - rim_sh).max(0.0);
            let r_base = (r0 * shadow_factor - rim_sh * sh_col[0] * 0.5).max(0.0);
            let g_base = (g0 * shadow_factor - rim_sh * sh_col[1] * 0.5).max(0.0);
            let b_base = (b0 * shadow_factor - rim_sh * sh_col[2] * 0.5).max(0.0);

            // Apply reception mode
            match reception_mode {
                1 => {
                    // Composite / Blend
                    let r_out = (r_base * (1.0 - l_rgb[0].min(1.0)) + l_rgb[0]).max(0.0);
                    let g_out = (g_base * (1.0 - l_rgb[1].min(1.0)) + l_rgb[1]).max(0.0);
                    let b_out = (b_base * (1.0 - l_rgb[2].min(1.0)) + l_rgb[2]).max(0.0);
                    let light_max = (l_rgb[0] * 0.299 + l_rgb[1] * 0.587 + l_rgb[2] * 0.114).min(1.0);
                    let a_out = (a + light_max * (1.0 - a)).clamp(0.0, 1.0);

                    c[0] = r_out * a_out;
                    c[1] = g_out * a_out;
                    c[2] = b_out * a_out;
                    c[3] = a_out;
                }
                2 => {
                    // Cutout (Light Only)
                    let lum = (l_rgb[0] * 0.2126 + l_rgb[1] * 0.7152 + l_rgb[2] * 0.0722).min(1.0);
                    let a_cut = (lum * a).clamp(0.0, 1.0);
                    c[0] = l_rgb[0] * a_cut;
                    c[1] = l_rgb[1] * a_cut;
                    c[2] = l_rgb[2] * a_cut;
                    c[3] = a_cut;
                }
                3 => {
                    // Screen
                    let r_out = 1.0 - (1.0 - r_base) * (1.0 - l_rgb[0].min(1.0));
                    let g_out = 1.0 - (1.0 - g_base) * (1.0 - l_rgb[1].min(1.0));
                    let b_out = 1.0 - (1.0 - b_base) * (1.0 - l_rgb[2].min(1.0));
                    c[0] = r_out * a;
                    c[1] = g_out * a;
                    c[2] = b_out * a;
                }
                _ => {
                    // Add
                    c[0] = (r_base + l_rgb[0]) * a;
                    c[1] = (g_base + l_rgb[1]) * a;
                    c[2] = (b_base + l_rgb[2]) * a;
                }
            }
        }
    }

    Ok(())
}

/// # Safety
/// The host passes pointers into the buffer `ec_alloc` returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ec_render(
    pixels: i32,
    width: i32,
    height: i32,
    params: i32,
    nparams: i32,
    time: f64,
    scale: f64,
) -> i32 {
    if width <= 0 || height <= 0 || nparams < 20 {
        return 1;
    }

    let p = unsafe { std::slice::from_raw_parts(params as usize as *const f64, nparams as usize) };
    let total_pixels = (width as usize) * (height as usize);
    let px = unsafe { std::slice::from_raw_parts_mut(pixels as usize as *mut [f32; 4], total_pixels) };

    if render_frame(px, width as usize, height as usize, p, time, scale).is_ok() {
        0
    } else {
        1
    }
}
