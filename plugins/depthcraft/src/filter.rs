//! Depth filtering, edge-preserving bilateral refinement, remapping, and visualization modes.

/// Remap raw depth in [0, 1] through near/far clipping, inversion, and gamma curve.
#[inline]
pub fn remap_depth(raw: f32, near: f32, far: f32, invert: bool, gamma: f32) -> f32 {
    let near_val = (near / 100.0).clamp(0.0, 1.0);
    let far_val = (far / 100.0).clamp(0.0, 1.0);
    let span = (far_val - near_val).abs().max(1e-5);

    let mut val = if far_val >= near_val { (raw - near_val) / span } else { (near_val - raw) / span };
    val = val.clamp(0.0, 1.0);

    if invert {
        val = 1.0 - val;
    }

    if gamma > 0.0 && (gamma - 1.0).abs() > 1e-4 {
        val = val.powf(1.0 / gamma.clamp(0.1, 10.0));
    }

    val.clamp(0.0, 1.0)
}

/// Fast 5x5 cross-bilateral filter that refines depth edges using image luminance guidance.
/// Aligns depth boundaries with object contours, preventing depth bleed.
pub fn refine_depth_edges(depth: &[f32], rgb: &[[f32; 3]], width: usize, height: usize, radius: usize, out: &mut [f32]) {
    if width == 0 || height == 0 || depth.len() < width * height || rgb.len() < width * height || out.len() < width * height {
        return;
    }

    if radius == 0 {
        out[..width * height].copy_from_slice(&depth[..width * height]);
        return;
    }

    let r = radius.min(4) as isize;
    let spatial_sigma = (r as f32) * 0.75;
    let inv_spatial_2sigma2 = 1.0 / (2.0 * spatial_sigma * spatial_sigma).max(1e-4);
    let range_sigma = 0.15f32;
    let inv_range_2sigma2 = 1.0 / (2.0 * range_sigma * range_sigma);

    for y in 0..height {
        let y_isize = y as isize;
        let row_start = y * width;

        for x in 0..width {
            let x_isize = x as isize;
            let center_idx = row_start + x;

            let center_rgb = match rgb.get(center_idx) {
                Some(c) => *c,
                None => continue,
            };
            let center_lum = 0.2126 * center_rgb[0] + 0.7152 * center_rgb[1] + 0.0722 * center_rgb[2];
            let center_d = match depth.get(center_idx) {
                Some(d) => *d,
                None => continue,
            };

            let mut sum_weight = 0.0f32;
            let mut sum_depth = 0.0f32;

            for dy in -r..=r {
                let ny = y_isize + dy;
                if ny < 0 || ny >= height as isize {
                    continue;
                }
                let ny_idx = (ny as usize) * width;

                for dx in -r..=r {
                    let nx = x_isize + dx;
                    if nx < 0 || nx >= width as isize {
                        continue;
                    }
                    let neighbor_idx = ny_idx + (nx as usize);

                    let n_rgb = match rgb.get(neighbor_idx) {
                        Some(c) => *c,
                        None => continue,
                    };
                    let n_lum = 0.2126 * n_rgb[0] + 0.7152 * n_rgb[1] + 0.0722 * n_rgb[2];
                    let n_d = match depth.get(neighbor_idx) {
                        Some(d) => *d,
                        None => continue,
                    };

                    let spatial_dist_sq = (dx * dx + dy * dy) as f32;
                    let lum_diff = (n_lum - center_lum).abs();

                    let spatial_w = (-spatial_dist_sq * inv_spatial_2sigma2).exp();
                    let range_w = (-lum_diff * lum_diff * inv_range_2sigma2).exp();
                    let w = spatial_w * range_w;

                    sum_weight += w;
                    sum_depth += w * n_d;
                }
            }

            if sum_weight > 1e-6 {
                if let Some(target) = out.get_mut(center_idx) {
                    *target = sum_depth / sum_weight;
                }
            } else if let Some(target) = out.get_mut(center_idx) {
                *target = center_d;
            }
        }
    }
}

/// Maps normalized depth in [0, 1] to a high-contrast heatmap (Turbo colormap approximation).
/// Smooth perceptual gradient from near (warm reds/yellows) to far (cool blues/purples).
#[inline]
pub fn turbo_colormap(t: f32) -> [f32; 3] {
    let x = t.clamp(0.0, 1.0);

    // Polynomial approximation of Google's Turbo colormap (Anton Mikhailov, 2019)
    let r = (0.1357 + x * (4.5974 - x * (42.3277 - x * (130.5887 - x * (150.5667 - x * 58.1375))))).clamp(0.0, 1.0);
    let g = (0.0914 + x * (2.1856 + x * (4.8052 - x * (14.0195 - x * (4.2109 - x * 2.7747))))).clamp(0.0, 1.0);
    let b = (0.1067 + x * (12.5833 - x * (78.4357 - x * (207.6083 - x * (244.9818 - x * 106.6667))))).clamp(0.0, 1.0);

    [r, g, b]
}

/// Blends depth-based volumetric atmospheric fog over source RGB pixels.
#[inline]
pub fn blend_fog(src_rgb: [f32; 3], depth: f32, density: f32, fog_color: [f32; 3]) -> [f32; 3] {
    let d = (depth * (density / 100.0)).clamp(0.0, 1.0);
    [src_rgb[0] * (1.0 - d) + fog_color[0] * d, src_rgb[1] * (1.0 - d) + fog_color[1] * d, src_rgb[2] * (1.0 - d) + fog_color[2] * d]
}

/// Generates depth slice / isoline highlight.
#[inline]
pub fn depth_slice_highlight(depth: f32, center: f32, width: f32) -> f32 {
    let c = (center / 100.0).clamp(0.0, 1.0);
    let w = (width / 100.0).clamp(0.01, 1.0);
    let dist = (depth - c).abs() / w;
    if dist <= 1.0 { 1.0 - dist } else { 0.0 }
}
