//! Glassify: Advanced optical glass rendering effect plug-in for EffectCraft.
//!
//! Complies with EffectCraft Effect Plug-in API v1 (see `docs/plugins.md`).
//! Simulates realistic glass surfaces with:
//! - Snell's Law refraction and displacement vectors
//! - Microfacet frosted roughness blur (Draft, Good, Cinematic)
//! - Cauchy chromatic dispersion splitting (RGB spectral fringe)
//! - Schlick Fresnel reflection at grazing angles
//! - Blinn-Phong specular highlights from directional key light
//! - Beer-Lambert volumetric absorption and edge rim tint
//! - Multiple procedural distortion generators (Liquid Waves, Frosted Grain, Caustics, Ribbed/Fluted)
//! - Edge shaping profiles (Convex, Chisel, Flat UI Pane, Cylinder Rim)
//! - Transparent background cutout and layer refraction modes

pub mod distortion;
pub mod optics;
pub mod presets;

use std::cell::RefCell;

use distortion::DistortionType;
use optics::{
    CompositeMode, HeightField, HeightSource, RenderQuality, ShapeProfile,
    apply_beer_lambert_tint, blinn_phong_specular, fresnel_schlick,
    sample_refracted_with_dispersion_and_frosted,
};
use presets::{PresetConfig, PresetId};

pub const MANIFEST: &str = r#"{
  "api": 1,
  "id": "org.effectcraft.plugins.glassify",
  "name": "Glassify",
  "category": "Distort",
  "version": "1.0.0",
  "author": "EffectCraft contributors",
  "description": "Optical glass rendering with Snell refraction, chromatic dispersion, frosted blur, and procedural liquid/caustic distortion.",
  "params": [
    {"id": "preset", "name": "Preset", "type": "popup", "options": ["Custom", "Lucid", "Frosted / Satin", "Liquid Glass", "Onyx Smoke", "Prismatic Crystal", "Ribbed / Fluted"], "default": 0},
    {"id": "refraction", "name": "Refraction Strength", "type": "slider", "default": 40.0, "min": 0.0, "max": 200.0, "sliderMax": 100.0, "decimals": 1},
    {"id": "ior", "name": "Index of Refraction (IOR)", "type": "slider", "default": 1.52, "min": 1.0, "max": 3.0, "sliderMin": 1.0, "sliderMax": 2.5, "decimals": 2},
    {"id": "thickness", "name": "Glass Thickness", "type": "slider", "default": 18.0, "min": 1.0, "max": 100.0, "sliderMax": 50.0, "decimals": 1},
    {"id": "shape_profile", "name": "Glass Shape Profile", "type": "popup", "options": ["Smooth Convex", "Chisel Bevel", "Flat Pane / UI", "Cylinder Rim"], "default": 0},
    {"id": "edge_softness", "name": "Edge Softness", "type": "slider", "default": 5.0, "min": 0.5, "max": 50.0, "sliderMax": 25.0, "decimals": 1},
    {"id": "roughness", "name": "Surface Roughness (Frosted)", "type": "slider", "default": 0.0, "min": 0.0, "max": 100.0, "decimals": 1},
    {"id": "highlight_intensity", "name": "Highlight Intensity", "type": "slider", "default": 120.0, "min": 0.0, "max": 300.0, "sliderMax": 200.0, "decimals": 1},
    {"id": "light_angle", "name": "Light Angle", "type": "angle", "default": 315.0},
    {"id": "light_elevation", "name": "Light Elevation", "type": "slider", "default": 45.0, "min": 10.0, "max": 90.0, "decimals": 1},
    {"id": "light_color", "name": "Light Color", "type": "color", "default": [1.0, 1.0, 1.0, 1.0]},
    {"id": "fresnel_reflection", "name": "Fresnel Reflection", "type": "slider", "default": 45.0, "min": 0.0, "max": 100.0, "decimals": 1},
    {"id": "tint_color", "name": "Glass Tint Color", "type": "color", "default": [0.96, 0.98, 1.0, 1.0]},
    {"id": "tint_intensity", "name": "Body Tint Intensity", "type": "slider", "default": 10.0, "min": 0.0, "max": 100.0, "decimals": 1},
    {"id": "edge_tint_intensity", "name": "Edge Tint Intensity", "type": "slider", "default": 30.0, "min": 0.0, "max": 100.0, "decimals": 1},
    {"id": "chromatic_dispersion", "name": "Chromatic Dispersion", "type": "slider", "default": 15.0, "min": 0.0, "max": 100.0, "decimals": 1},
    {"id": "distortion_type", "name": "Distortion Type", "type": "popup", "options": ["None", "Liquid Waves", "Frosted Grain", "Caustics", "Ribbed / Fluted"], "default": 0},
    {"id": "distortion_scale", "name": "Distortion Scale", "type": "slider", "default": 40.0, "min": 5.0, "max": 200.0, "sliderMax": 100.0, "decimals": 1},
    {"id": "distortion_amount", "name": "Distortion Amount", "type": "slider", "default": 0.0, "min": 0.0, "max": 100.0, "decimals": 1},
    {"id": "distortion_speed", "name": "Distortion Speed", "type": "slider", "default": 0.0, "min": -5.0, "max": 5.0, "decimals": 2},
    {"id": "height_source", "name": "Height / Mask Source", "type": "popup", "options": ["Alpha (Logos & Text)", "Luminance (Photos & Art)", "Combined (Alpha + Luma)"], "default": 0},
    {"id": "composite_mode", "name": "Composite Mode", "type": "popup", "options": ["Refract Layer", "Cutout (Glass Only)", "Highlights Only (Screen)"], "default": 0},
    {"id": "quality", "name": "Render Quality", "type": "popup", "options": ["Draft (Fast)", "Good (Balanced)", "Cinematic (High Quality)"], "default": 1}
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
        0.0,                  // 0: preset (Custom)
        40.0,                 // 1: refraction
        1.52,                 // 2: ior
        18.0,                 // 3: thickness
        0.0,                  // 4: shape_profile (Smooth Convex)
        5.0,                  // 5: edge_softness
        0.0,                  // 6: roughness
        120.0,                // 7: highlight_intensity
        315.0,                // 8: light_angle
        45.0,                 // 9: light_elevation
        1.0, 1.0, 1.0, 1.0,   // 10..13: light_color
        45.0,                 // 14: fresnel_reflection
        0.96, 0.98, 1.0, 1.0, // 15..18: tint_color
        10.0,                 // 19: tint_intensity
        30.0,                 // 20: edge_tint_intensity
        15.0,                 // 21: chromatic_dispersion
        0.0,                  // 22: distortion_type (None)
        40.0,                 // 23: distortion_scale
        0.0,                  // 24: distortion_amount
        0.0,                  // 25: distortion_speed
        0.0,                  // 26: height_source (Alpha)
        0.0,                  // 27: composite_mode (Refract Layer)
        1.0,                  // 28: quality (Good)
    ]
}

/// Core rendering logic for Glassify, processing pixels in-place.
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

    // Parse parameters
    let preset_idx = params.first().copied().unwrap_or(0.0).round().max(0.0) as usize;
    let preset_id = PresetId::from_index(preset_idx);
    let preset_cfg = PresetConfig::for_preset(preset_id);

    let mut refraction = params.get(1).copied().unwrap_or(40.0) as f32;
    let mut ior = params.get(2).copied().unwrap_or(1.52) as f32;
    let mut thickness = params.get(3).copied().unwrap_or(18.0) as f32;
    let mut shape_profile = params.get(4).copied().unwrap_or(0.0).round().max(0.0) as usize;
    let mut edge_softness = params.get(5).copied().unwrap_or(5.0) as f32;
    let mut roughness = params.get(6).copied().unwrap_or(0.0) as f32;
    let mut highlight_intensity = params.get(7).copied().unwrap_or(120.0) as f32;
    let light_angle = params.get(8).copied().unwrap_or(315.0) as f32;
    let light_elevation = params.get(9).copied().unwrap_or(45.0) as f32;
    let light_color = [
        params.get(10).copied().unwrap_or(1.0) as f32,
        params.get(11).copied().unwrap_or(1.0) as f32,
        params.get(12).copied().unwrap_or(1.0) as f32,
        params.get(13).copied().unwrap_or(1.0) as f32,
    ];
    let mut fresnel_reflection = params.get(14).copied().unwrap_or(45.0) as f32;
    let mut tint_color = [
        params.get(15).copied().unwrap_or(0.96) as f32,
        params.get(16).copied().unwrap_or(0.98) as f32,
        params.get(17).copied().unwrap_or(1.0) as f32,
        params.get(18).copied().unwrap_or(1.0) as f32,
    ];
    let mut tint_intensity = params.get(19).copied().unwrap_or(10.0) as f32;
    let mut edge_tint_intensity = params.get(20).copied().unwrap_or(30.0) as f32;
    let mut chromatic_dispersion = params.get(21).copied().unwrap_or(15.0) as f32;
    let mut distortion_type = params.get(22).copied().unwrap_or(0.0).round().max(0.0) as usize;
    let mut distortion_scale = params.get(23).copied().unwrap_or(40.0) as f32;
    let mut distortion_amount = params.get(24).copied().unwrap_or(0.0) as f32;
    let mut distortion_speed = params.get(25).copied().unwrap_or(0.0) as f32;
    let height_source = params.get(26).copied().unwrap_or(0.0).round().max(0.0) as usize;
    let composite_mode = params.get(27).copied().unwrap_or(0.0).round().max(0.0) as usize;
    let quality = params.get(28).copied().unwrap_or(1.0).round().max(0.0) as usize;

    // Apply preset overrides if active
    if let Some(cfg) = preset_cfg {
        refraction = cfg.refraction;
        ior = cfg.ior;
        thickness = cfg.thickness;
        shape_profile = cfg.shape_profile;
        roughness = cfg.roughness;
        edge_softness = cfg.edge_softness;
        highlight_intensity = cfg.highlight_intensity;
        fresnel_reflection = cfg.fresnel_reflection;
        tint_color = cfg.tint_color;
        tint_intensity = cfg.tint_intensity;
        edge_tint_intensity = cfg.edge_tint_intensity;
        chromatic_dispersion = cfg.chromatic_dispersion;
        distortion_type = cfg.distortion_type;
        distortion_scale = cfg.distortion_scale;
        distortion_amount = cfg.distortion_amount;
        distortion_speed = cfg.distortion_speed;
    }

    // Resolution-aware scaling
    let thickness_px = (thickness * resolution_scale).max(0.5);
    let edge_softness_px = (edge_softness * resolution_scale).max(0.5);
    let distortion_scale_px = (distortion_scale * resolution_scale).max(1.0);
    let roughness_radius_px = (roughness / 100.0) * 18.0 * resolution_scale;

    // Light directional unit vector
    let angle_rad = light_angle.to_radians();
    let elev_rad = light_elevation.clamp(5.0, 90.0).to_radians();
    let lx = elev_rad.cos() * angle_rad.cos();
    let ly = elev_rad.cos() * angle_rad.sin();
    let lz = elev_rad.sin();
    let light_dir = (lx, ly, lz);

    let src_copy = px[0..width * height].to_vec();

    // Precompute heightfield from chosen source
    let h_src = HeightSource::from_index(height_source);
    let s_prof = ShapeProfile::from_index(shape_profile);
    let heightfield = HeightField::build(&src_copy, width, height, h_src, edge_softness_px, s_prof);

    let dist_type = DistortionType::from_index(distortion_type);
    let comp_mode = CompositeMode::from_index(composite_mode);
    let render_qual = RenderQuality::from_index(quality);

    for y in 0..height {
        let row_offset = y * width;
        for x in 0..width {
            let idx = row_offset + x;
            let h = heightfield.get(x, y);

            if h <= 0.0001 {
                match comp_mode {
                    CompositeMode::CutoutGlass | CompositeMode::HighlightsOnly => {
                        px[idx] = [0.0, 0.0, 0.0, 0.0];
                    }
                    CompositeMode::RefractLayer => {
                        // Original pixel remains untouched
                    }
                }
                continue;
            }

            // Surface normal combining height gradient and procedural distortion
            let normal = heightfield.surface_normal(
                x,
                y,
                thickness_px,
                dist_type,
                distortion_scale_px,
                distortion_amount,
                distortion_speed,
                time as f32,
            );
            let (nx, ny, nz) = normal;

            // Optical refraction with optional Cauchy dispersion & frosted roughness
            let refracted = sample_refracted_with_dispersion_and_frosted(
                &src_copy,
                width,
                height,
                x as f32,
                y as f32,
                nx,
                ny,
                ior,
                thickness_px,
                refraction,
                chromatic_dispersion,
                roughness_radius_px,
                render_qual,
            );

            // Volumetric Beer-Lambert absorption and edge tinting
            let tinted = apply_beer_lambert_tint(
                refracted,
                tint_color,
                tint_intensity,
                edge_tint_intensity,
                nz,
            );

            // Schlick Fresnel reflection
            let fresnel = fresnel_schlick(nz, ior) * (fresnel_reflection / 100.0) * 1.5;

            // Blinn-Phong specular highlight
            let spec = blinn_phong_specular(normal, light_dir, roughness) * (highlight_intensity / 100.0);

            // Reflected component: specular point + Fresnel rim tint
            let reflected_r = light_color[0] * spec + tint_color[0] * fresnel * 0.7;
            let reflected_g = light_color[1] * spec + tint_color[1] * fresnel * 0.7;
            let reflected_b = light_color[2] * spec + tint_color[2] * fresnel * 0.7;
            let reflected_a = (spec + fresnel).clamp(0.0, 1.0);

            let orig = src_copy[idx];

            match comp_mode {
                CompositeMode::RefractLayer => {
                    let glass_r = (tinted[0] * (1.0 - fresnel * 0.35) + reflected_r).max(0.0);
                    let glass_g = (tinted[1] * (1.0 - fresnel * 0.35) + reflected_g).max(0.0);
                    let glass_b = (tinted[2] * (1.0 - fresnel * 0.35) + reflected_b).max(0.0);
                    let glass_a = tinted[3].max(orig[3]);

                    // Smooth edge blend with background based on heightfield mask h
                    let r_out = orig[0] * (1.0 - h) + glass_r * h;
                    let g_out = orig[1] * (1.0 - h) + glass_g * h;
                    let b_out = orig[2] * (1.0 - h) + glass_b * h;
                    let a_out = orig[3] * (1.0 - h) + glass_a * h;

                    px[idx] = [r_out, g_out, b_out, a_out];
                }
                CompositeMode::CutoutGlass => {
                    let r_out = (tinted[0] + reflected_r).max(0.0) * h;
                    let g_out = (tinted[1] + reflected_g).max(0.0) * h;
                    let b_out = (tinted[2] + reflected_b).max(0.0) * h;
                    let a_out = (tinted[3] * h).clamp(0.0, 1.0);

                    px[idx] = [r_out, g_out, b_out, a_out];
                }
                CompositeMode::HighlightsOnly => {
                    let r_out = reflected_r * h;
                    let g_out = reflected_g * h;
                    let b_out = reflected_b * h;
                    let a_out = reflected_a * h;

                    px[idx] = [r_out, g_out, b_out, a_out];
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
