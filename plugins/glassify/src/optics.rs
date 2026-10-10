//! Optical simulation algorithms and surface shading for Glassify.
//!
//! Implements:
//! - Physical 2.5D heightfield generation from alpha/luminance/hybrid channels.
//! - Curvature and shape profile remapping (Convex, Chisel, Flat UI Pane, Cylinder Rim).
//! - Surface normal estimation with procedural distortion modulation.
//! - Snell's Law refraction vector computation.
//! - Cauchy chromatic dispersion (wavelength-dependent split).
//! - Jittered multi-tap microfacet frosted blur (Draft/Good/Cinematic).
//! - Schlick Fresnel reflection calculation.
//! - Blinn-Phong specular lighting.
//! - Beer-Lambert volumetric absorption and edge tint.

use std::f32::consts::PI;
use crate::distortion::{DistortionType, sample_distortion_gradient};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightSource {
    Alpha = 0,
    Luminance = 1,
    Combined = 2,
}

impl HeightSource {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => HeightSource::Luminance,
            2 => HeightSource::Combined,
            _ => HeightSource::Alpha,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeProfile {
    SmoothConvex = 0,
    ChiselBevel = 1,
    FlatPaneUI = 2,
    CylinderRim = 3,
}

impl ShapeProfile {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => ShapeProfile::ChiselBevel,
            2 => ShapeProfile::FlatPaneUI,
            3 => ShapeProfile::CylinderRim,
            _ => ShapeProfile::SmoothConvex,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderQuality {
    Draft = 0,
    Good = 1,
    Cinematic = 2,
}

impl RenderQuality {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => RenderQuality::Draft,
            2 => RenderQuality::Cinematic,
            _ => RenderQuality::Good,
        }
    }

    pub fn sample_count(&self) -> usize {
        match self {
            RenderQuality::Draft => 4,
            RenderQuality::Good => 8,
            RenderQuality::Cinematic => 16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeMode {
    RefractLayer = 0,
    CutoutGlass = 1,
    HighlightsOnly = 2,
}

impl CompositeMode {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => CompositeMode::CutoutGlass,
            2 => CompositeMode::HighlightsOnly,
            _ => CompositeMode::RefractLayer,
        }
    }
}

/// Precomputed heightfield with smooth edge falloff.
pub struct HeightField {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

impl HeightField {
    pub fn build(
        pixels: &[[f32; 4]],
        width: usize,
        height: usize,
        source: HeightSource,
        softness_px: f32,
        profile: ShapeProfile,
    ) -> Self {
        let total = width * height;
        let mut raw = vec![0.0f32; total];

        // 1. Extract raw height from selected channel
        for i in 0..total {
            let px = pixels[i];
            let alpha = px[3].clamp(0.0, 1.0);
            let lum = (0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2]).clamp(0.0, 1.0);

            let val = match source {
                HeightSource::Alpha => alpha,
                HeightSource::Luminance => lum,
                HeightSource::Combined => alpha * (0.25 + 0.75 * lum),
            };
            raw[i] = val;
        }

        // 2. Separable box blur pass for continuous edge softness and smooth normals
        let blur_r = (softness_px.round() as isize).max(1);
        let mut blurred_h = vec![0.0f32; total];
        let mut blurred_final = vec![0.0f32; total];

        // Horizontal pass
        for y in 0..height {
            let row_offset = y * width;
            let mut sum = 0.0f32;
            let mut count = 0;

            for x in 0..width {
                if x == 0 {
                    for k in -blur_r..=blur_r {
                        let nx = k.clamp(0, (width - 1) as isize) as usize;
                        sum += raw[row_offset + nx];
                        count += 1;
                    }
                } else {
                    let old_x = (x as isize - blur_r - 1).clamp(0, (width - 1) as isize) as usize;
                    let new_x = (x as isize + blur_r).clamp(0, (width - 1) as isize) as usize;
                    sum += raw[row_offset + new_x] - raw[row_offset + old_x];
                }
                blurred_h[row_offset + x] = sum / (count as f32);
            }
        }

        // Vertical pass
        for x in 0..width {
            let mut sum = 0.0f32;
            let mut count = 0;

            for y in 0..height {
                if y == 0 {
                    for k in -blur_r..=blur_r {
                        let ny = k.clamp(0, (height - 1) as isize) as usize;
                        sum += blurred_h[ny * width + x];
                        count += 1;
                    }
                } else {
                    let old_y = (y as isize - blur_r - 1).clamp(0, (height - 1) as isize) as usize;
                    let new_y = (y as isize + blur_r).clamp(0, (height - 1) as isize) as usize;
                    sum += blurred_h[new_y * width + x] - blurred_h[old_y * width + x];
                }
                blurred_final[y * width + x] = sum / (count as f32);
            }
        }

        // 3. Remap height using ShapeProfile
        for val in blurred_final.iter_mut() {
            let h = (*val).clamp(0.0, 1.0);
            *val = match profile {
                ShapeProfile::SmoothConvex => {
                    // Spherical/dome curve
                    (h * (PI * 0.5)).sin()
                }
                ShapeProfile::ChiselBevel => {
                    // Crisp linear bevel
                    if h < 0.5 { h * 2.0 } else { 1.0 }
                }
                ShapeProfile::FlatPaneUI => {
                    // Smooth curved border rim, completely flat center
                    if h < 0.95 {
                        let t = h / 0.95;
                        t * t * (3.0 - 2.0 * t) * 0.95
                    } else {
                        1.0
                    }
                }
                ShapeProfile::CylinderRim => {
                    // Elevated perimeter ridge
                    (4.0 * h * (1.0 - h)).clamp(0.0, 1.0)
                }
            };
        }

        HeightField {
            width,
            height,
            data: blurred_final,
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.data[y * self.width + x]
    }

    /// Evaluates surface normal at (x, y), combining height gradient and procedural distortion.
    pub fn surface_normal(
        &self,
        x: usize,
        y: usize,
        thickness_px: f32,
        dist_type: DistortionType,
        dist_scale: f32,
        dist_amount: f32,
        dist_speed: f32,
        time: f32,
    ) -> (f32, f32, f32) {
        let x_left = x.saturating_sub(1);
        let x_right = (x + 1).min(self.width - 1);
        let y_top = y.saturating_sub(1);
        let y_bot = (y + 1).min(self.height - 1);

        let h_left = self.get(x_left, y);
        let h_right = self.get(x_right, y);
        let h_top = self.get(x, y_top);
        let h_bot = self.get(x, y_bot);

        let dx_denom = (x_right - x_left).max(1) as f32;
        let dy_denom = (y_bot - y_top).max(1) as f32;

        let grad_x = (h_right - h_left) / dx_denom;
        let grad_y = (h_bot - h_top) / dy_denom;

        // Scale gradient by thickness factor
        let factor = (thickness_px * 0.25).max(0.1);
        let mut sx = -grad_x * factor;
        let mut sy = -grad_y * factor;

        // Add procedural distortion gradient if enabled
        if dist_type != DistortionType::None && dist_amount > 0.001 {
            let (dgx, dgy) = sample_distortion_gradient(
                x as f32,
                y as f32,
                time,
                dist_type,
                dist_scale,
                dist_amount,
                dist_speed,
            );
            sx += dgx * factor * 0.75;
            sy += dgy * factor * 0.75;
        }

        let sz = 1.0f32;
        let len = (sx * sx + sy * sy + sz * sz).sqrt();
        (sx / len, sy / len, sz / len)
    }
}

/// Bilinear interpolation sampler with boundary clamping.
#[inline]
pub fn sample_bilinear(pixels: &[[f32; 4]], width: usize, height: usize, u: f32, v: f32) -> [f32; 4] {
    let cu = u.clamp(0.0, (width - 1) as f32);
    let cv = v.clamp(0.0, (height - 1) as f32);

    let x0 = cu.floor() as usize;
    let y0 = cv.floor() as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = cu - x0 as f32;
    let fy = cv - y0 as f32;

    let p00 = pixels[y0 * width + x0];
    let p10 = pixels[y0 * width + x1];
    let p01 = pixels[y1 * width + x0];
    let p11 = pixels[y1 * width + x1];

    let w00 = (1.0 - fx) * (1.0 - fy);
    let w10 = fx * (1.0 - fy);
    let w01 = (1.0 - fx) * fy;
    let w11 = fx * fy;

    [
        p00[0] * w00 + p10[0] * w10 + p01[0] * w01 + p11[0] * w11,
        p00[1] * w00 + p10[1] * w10 + p01[1] * w01 + p11[1] * w11,
        p00[2] * w00 + p10[2] * w10 + p01[2] * w01 + p11[2] * w11,
        p00[3] * w00 + p10[3] * w10 + p01[3] * w01 + p11[3] * w11,
    ]
}

/// Evaluates Snell refraction deflection vector in 2.5D screen space:
/// D = N_xy * (1 - 1 / eta) * thickness * scale * (refraction / 40.0)
#[inline]
pub fn snell_displacement(
    nx: f32,
    ny: f32,
    ior: f32,
    thickness_px: f32,
    refraction_strength: f32,
) -> (f32, f32) {
    let safe_ior = ior.max(1.001);
    let snell_factor = 1.0 - (1.0 / safe_ior);
    let mag = snell_factor * thickness_px * (refraction_strength / 40.0);
    (nx * mag, ny * mag)
}

/// Evaluates Schlick's approximation for Fresnel reflection:
/// R(theta) = R0 + (1 - R0) * (1 - cos theta)^5
#[inline]
pub fn fresnel_schlick(nz: f32, ior: f32) -> f32 {
    let safe_ior = ior.max(1.001);
    let r0_term = (safe_ior - 1.0) / (safe_ior + 1.0);
    let r0 = r0_term * r0_term;

    let cos_theta = nz.clamp(0.0, 1.0);
    let grazing = (1.0 - cos_theta).powi(5);
    r0 + (1.0 - r0) * grazing
}

/// Blinn-Phong specular highlight for a given normal and light source.
#[inline]
pub fn blinn_phong_specular(
    normal: (f32, f32, f32),
    light_dir: (f32, f32, f32),
    roughness: f32,
) -> f32 {
    let (nx, ny, nz) = normal;
    let (lx, ly, lz) = light_dir;

    // View vector is pointing directly out along +Z (0, 0, 1)
    let vx = 0.0f32;
    let vy = 0.0f32;
    let vz = 1.0f32;

    // Half vector H = (L + V) / |L + V|
    let hx = lx + vx;
    let hy = ly + vy;
    let hz = lz + vz;
    let h_len = (hx * hx + hy * hy + hz * hz).sqrt().max(0.0001);
    let (hx, hy, hz) = (hx / h_len, hy / h_len, hz / h_len);

    let n_dot_h = (nx * hx + ny * hy + nz * hz).max(0.0);

    // Shininess decreases as roughness increases
    let rough_norm = (roughness / 100.0).clamp(0.0, 1.0);
    let shininess = 120.0 * (1.0 - rough_norm * 0.85).powi(2) + 4.0;

    n_dot_h.powf(shininess)
}

/// Vogel spiral disk offsets for jittered multi-tap frosted blur.
#[inline]
fn vogel_offset(index: usize, total_taps: usize, radius: f32) -> (f32, f32) {
    let golden_angle = 2.39996323f32; // Golden angle in radians
    let r = ((index as f32 + 0.5) / (total_taps as f32)).sqrt() * radius;
    let theta = (index as f32) * golden_angle;
    (r * theta.cos(), r * theta.sin())
}

/// Samples the source image at the refracted point with optional chromatic dispersion
/// and frosted microfacet roughness blur.
pub fn sample_refracted_with_dispersion_and_frosted(
    src: &[[f32; 4]],
    width: usize,
    height: usize,
    px: f32,
    py: f32,
    nx: f32,
    ny: f32,
    ior: f32,
    thickness_px: f32,
    refraction_strength: f32,
    chromatic_dispersion: f32,
    roughness_radius_px: f32,
    quality: RenderQuality,
) -> [f32; 4] {
    let dispersion_delta = (chromatic_dispersion / 100.0) * 0.12;

    // Red refracts less (lower IOR), Blue refracts more (higher IOR)
    let (dx_red, dy_red) = snell_displacement(
        nx,
        ny,
        (ior - dispersion_delta).max(1.001),
        thickness_px,
        refraction_strength,
    );
    let (dx_green, dy_green) = snell_displacement(
        nx,
        ny,
        ior,
        thickness_px,
        refraction_strength,
    );
    let (dx_blue, dy_blue) = snell_displacement(
        nx,
        ny,
        ior + dispersion_delta,
        thickness_px,
        refraction_strength,
    );

    // If roughness is near zero, perform single bilinear samples for R, G, B
    if roughness_radius_px <= 0.2 {
        let p_red = sample_bilinear(src, width, height, px + dx_red, py + dy_red);
        let p_green = sample_bilinear(src, width, height, px + dx_green, py + dy_green);
        let p_blue = sample_bilinear(src, width, height, px + dx_blue, py + dy_blue);

        return [p_red[0], p_green[1], p_blue[2], p_green[3]];
    }

    // Frosted microfacet blur: multi-tap Vogel disk sampling
    let taps = quality.sample_count();
    let inv_taps = 1.0 / (taps as f32);

    let mut sum_r = 0.0f32;
    let mut sum_g = 0.0f32;
    let mut sum_b = 0.0f32;
    let mut sum_a = 0.0f32;

    for i in 0..taps {
        let (ox, oy) = vogel_offset(i, taps, roughness_radius_px);

        let tap_red = sample_bilinear(src, width, height, px + dx_red + ox, py + dy_red + oy);
        let tap_green = sample_bilinear(src, width, height, px + dx_green + ox, py + dy_green + oy);
        let tap_blue = sample_bilinear(src, width, height, px + dx_blue + ox, py + dy_blue + oy);

        sum_r += tap_red[0];
        sum_g += tap_green[1];
        sum_b += tap_blue[2];
        sum_a += tap_green[3];
    }

    [
        sum_r * inv_taps,
        sum_g * inv_taps,
        sum_b * inv_taps,
        sum_a * inv_taps,
    ]
}

/// Applies Beer-Lambert volumetric absorption and edge tinting.
#[inline]
pub fn apply_beer_lambert_tint(
    base_color: [f32; 4],
    tint_color: [f32; 4],
    tint_intensity: f32,
    edge_tint_intensity: f32,
    nz: f32,
) -> [f32; 4] {
    let t_int = (tint_intensity / 100.0).clamp(0.0, 1.0);
    let e_int = (edge_tint_intensity / 100.0).clamp(0.0, 1.0);

    if t_int <= 0.001 && e_int <= 0.001 {
        return base_color;
    }

    // Path length through slab is inversely proportional to normal Nz
    let safe_nz = nz.clamp(0.1, 1.0);
    let path_factor = 1.0 / safe_nz;

    // Body tint: absorption increases with path length
    let body_absorption = 1.0 - (-t_int * 1.2 * path_factor).exp();
    // Edge tint: concentration at glancing angles where Nz -> 0
    let edge_rim = (1.0 - safe_nz).powi(2) * e_int;

    let total_tint = (body_absorption + edge_rim).clamp(0.0, 0.98);

    // Apply color multiplication and tint lerp
    let r = base_color[0] * (1.0 - total_tint) + (base_color[0] * tint_color[0]) * total_tint;
    let g = base_color[1] * (1.0 - total_tint) + (base_color[1] * tint_color[1]) * total_tint;
    let b = base_color[2] * (1.0 - total_tint) + (base_color[2] * tint_color[2]) * total_tint;

    [r, g, b, base_color[3]]
}
