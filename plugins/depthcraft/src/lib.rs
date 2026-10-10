//! DepthCraft: Monocular depth estimation and depth-map processing plugin for EffectCraft.
//!
//! Complies with EffectCraft Effect Plug-in API v1 (see `docs/plugins.md`).
//! Generates grayscale depth maps, heatmaps, atmospheric fog, and depth slices
//! from still images and video frames.

pub mod estimator;
pub mod filter;
pub mod models;
pub mod pipeline;
pub mod temporal;

use std::cell::RefCell;

use estimator::estimate_depth_proxy;
use filter::{blend_fog, depth_slice_highlight, refine_depth_edges, remap_depth, turbo_colormap};
use temporal::apply_temporal_smoothing;

pub const MANIFEST: &str = r#"{
  "api": 1,
  "id": "org.effectcraft.plugins.depthcraft",
  "name": "DepthCraft",
  "category": "Stylize",
  "version": "1.0.0",
  "author": "EffectCraft contributors",
  "description": "Monocular depth estimation and depth-map processing with near/far remapping, curves, inversion, and temporal smoothing.",
  "params": [
    {"id": "mode", "name": "Mode", "type": "popup", "options": ["Grayscale Depth", "False Color (Heatmap)", "Atmospheric Fog", "Depth Slice (Isoline)", "Depth Cutout"], "default": 0},
    {"id": "near_depth", "name": "Near Depth", "type": "slider", "default": 0, "min": 0, "max": 100, "sliderMax": 100, "decimals": 1},
    {"id": "far_depth", "name": "Far Depth", "type": "slider", "default": 100, "min": 0, "max": 100, "sliderMax": 100, "decimals": 1},
    {"id": "invert", "name": "Invert Depth", "type": "checkbox", "default": false},
    {"id": "gamma", "name": "Depth Gamma", "type": "slider", "default": 1.0, "min": 0.1, "max": 5.0, "decimals": 2},
    {"id": "temporal_smooth", "name": "Temporal Smoothing", "type": "slider", "default": 0, "min": 0, "max": 100, "decimals": 0},
    {"id": "edge_refine", "name": "Edge Refinement", "type": "slider", "default": 2.0, "min": 0.0, "max": 20.0, "decimals": 1},
    {"id": "fog_density", "name": "Fog Density", "type": "slider", "default": 50, "min": 0, "max": 100, "decimals": 0},
    {"id": "fog_color", "name": "Fog Color", "type": "color", "default": [0.8, 0.85, 0.9, 1.0]},
    {"id": "slice_center", "name": "Slice Center", "type": "slider", "default": 50, "min": 0, "max": 100, "decimals": 1},
    {"id": "slice_width", "name": "Slice Width", "type": "slider", "default": 20, "min": 1, "max": 100, "decimals": 1}
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

/// Core rendering logic, processing pixels in-place.
pub fn render_frame(px: &mut [[f32; 4]], width: usize, height: usize, params: &[f64], time: f64) -> Result<(), &'static str> {
    if params.len() < 14 || width == 0 || height == 0 || px.len() < width * height {
        return Err("invalid frame dimensions or insufficient parameters");
    }

    let mode = params.first().copied().unwrap_or(0.0).round().max(0.0) as usize;
    let near = params.get(1).copied().unwrap_or(0.0) as f32;
    let far = params.get(2).copied().unwrap_or(100.0) as f32;
    let invert = params.get(3).copied().unwrap_or(0.0) != 0.0;
    let gamma = params.get(4).copied().unwrap_or(1.0) as f32;
    let temporal_smooth = params.get(5).copied().unwrap_or(0.0) as f32;
    let edge_refine = params.get(6).copied().unwrap_or(2.0).round().max(0.0) as usize;
    let fog_density = params.get(7).copied().unwrap_or(50.0) as f32;
    let fog_color =
        [params.get(8).copied().unwrap_or(0.8) as f32, params.get(9).copied().unwrap_or(0.85) as f32, params.get(10).copied().unwrap_or(0.9) as f32];
    let slice_center = params.get(12).copied().unwrap_or(50.0) as f32;
    let slice_width = params.get(13).copied().unwrap_or(20.0) as f32;

    let total_pixels = width * height;

    // Un-premultiply RGB and compute luminance
    let mut rgb = vec![[0.0f32; 3]; total_pixels];
    let mut lum = vec![0.0f32; total_pixels];
    for (i, c) in px.iter().take(total_pixels).enumerate() {
        let a = c[3];
        let (r, g, b) = if a > 1e-4 { (c[0] / a, c[1] / a, c[2] / a) } else { (c[0], c[1], c[2]) };
        if let Some(target) = rgb.get_mut(i) {
            *target = [r, g, b];
        }
        if let Some(target) = lum.get_mut(i) {
            *target = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        }
    }

    // Step 1: Initial depth estimate
    let mut depth = vec![0.0f32; total_pixels];
    estimate_depth_proxy(&rgb, width, height, &mut depth);

    // Step 2: Edge-preserving bilateral refinement
    if edge_refine > 0 {
        let mut refined = vec![0.0f32; total_pixels];
        refine_depth_edges(&depth, &rgb, width, height, edge_refine, &mut refined);
        depth = refined;
    }

    // Step 3: Temporal smoothing across frames for video
    if temporal_smooth > 0.0 {
        apply_temporal_smoothing(&mut depth, &lum, width, height, time, temporal_smooth);
    }

    // Step 4: Render to output pixels based on selected Mode
    for (i, c) in px.iter_mut().take(total_pixels).enumerate() {
        let a = c[3];
        if a <= 0.0 {
            continue;
        }

        let raw_d = depth.get(i).copied().unwrap_or(0.5);
        let val = remap_depth(raw_d, near, far, invert, gamma);
        let src_c = rgb.get(i).copied().unwrap_or([0.0; 3]);

        match mode {
            0 => {
                // Grayscale Depth Map
                c[0] = val * a;
                c[1] = val * a;
                c[2] = val * a;
            }
            1 => {
                // False Color (Heatmap)
                let map = turbo_colormap(val);
                c[0] = map[0] * a;
                c[1] = map[1] * a;
                c[2] = map[2] * a;
            }
            2 => {
                // Atmospheric Fog
                let fogged = blend_fog(src_c, val, fog_density, fog_color);
                c[0] = fogged[0] * a;
                c[1] = fogged[1] * a;
                c[2] = fogged[2] * a;
            }
            3 => {
                // Depth Slice (Isoline)
                let hl = depth_slice_highlight(val, slice_center, slice_width);
                let r = (src_c[0] * (1.0 - hl * 0.5) + hl * 0.2).min(1.0);
                let g = (src_c[1] * (1.0 - hl * 0.5) + hl * 1.0).min(1.0);
                let b = (src_c[2] * (1.0 - hl * 0.5) + hl * 0.8).min(1.0);
                c[0] = r * a;
                c[1] = g * a;
                c[2] = b * a;
            }
            4 => {
                // Depth Cutout
                let threshold = (slice_center / 100.0).clamp(0.0, 1.0);
                let mask = if val <= threshold { 1.0 } else { 0.0 };
                c[0] *= mask;
                c[1] *= mask;
                c[2] *= mask;
                c[3] *= mask;
            }
            _ => {
                // Fallback to Grayscale Depth
                c[0] = val * a;
                c[1] = val * a;
                c[2] = val * a;
            }
        }
    }

    Ok(())
}

/// # Safety
/// The host passes pointers into the buffer `ec_alloc` returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ec_render(pixels: i32, width: i32, height: i32, params: i32, nparams: i32, time: f64, _scale: f64) -> i32 {
    if nparams < 14 || width <= 0 || height <= 0 {
        return 1;
    }

    let p = unsafe { std::slice::from_raw_parts(params as usize as *const f64, nparams as usize) };
    let total_pixels = (width as usize) * (height as usize);
    let px = unsafe { std::slice::from_raw_parts_mut(pixels as usize as *mut [f32; 4], total_pixels) };

    if render_frame(px, width as usize, height as usize, p, time).is_ok() { 0 } else { 1 }
}
