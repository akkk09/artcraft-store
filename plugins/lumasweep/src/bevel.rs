//! Bevel-like edge response, surface normal construction, and luminance heightfield for LumaSweep.

#[derive(Debug, Clone, Copy)]
pub struct BevelConfig {
    pub intensity: f32,
    pub depth: f32,
    pub profile: usize,
    pub edge_source: usize,
    pub shadow_intensity: f32,
}

/// Precomputed surface heightfield and edge gradient buffer.
pub struct SurfaceField {
    pub width: usize,
    pub height: usize,
    pub smoothed: Vec<f32>,
}

impl SurfaceField {
    /// Construct smoothed surface field from pixels based on edge_source.
    ///
    /// * `edge_source`: 0 = Alpha, 1 = Luminance, 2 = Combined.
    /// * `depth`: Bevel depth in pixels (scaled by preview scale).
    pub fn build(
        pixels: &[[f32; 4]],
        width: usize,
        height: usize,
        edge_source: usize,
        depth: f32,
    ) -> Self {
        let total = width * height;
        let mut raw = vec![0.0f32; total];

        for (i, p) in pixels.iter().take(total).enumerate() {
            let a = p[3].clamp(0.0, 1.0);
            let (r, g, b) = if a > 1e-4 {
                (p[0] / a, p[1] / a, p[2] / a)
            } else {
                (p[0], p[1], p[2])
            };
            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;

            raw[i] = match edge_source {
                1 => lum.clamp(0.0, 1.0), // Luminance only (photos / artwork)
                2 => (a * (0.4 + 0.6 * lum)).clamp(0.0, 1.0), // Combined hybrid
                _ => a, // Alpha only (logos / text)
            };
        }

        // Fast separable smoothing pass to establish bevel slope
        let radius = (depth * 0.75).round().max(0.0) as usize;
        let smoothed = if radius > 0 {
            smooth_separable(&raw, width, height, radius)
        } else {
            raw
        };

        Self {
            width,
            height,
            smoothed,
        }
    }

    /// Calculate edge lighting contribution and rim shadow for a pixel at (x, y).
    ///
    /// Returns `(edge_highlight, rim_shadow)`.
    #[inline]
    pub fn evaluate_shading(
        &self,
        x: usize,
        y: usize,
        cfg: &BevelConfig,
        light_dir: (f32, f32),
    ) -> (f32, f32) {
        if cfg.intensity <= 0.001 && cfg.shadow_intensity <= 0.001 {
            return (0.0, 0.0);
        }

        let w = self.width;
        let h = self.height;

        let x_prev = x.saturating_sub(1);
        let x_next = (x + 1).min(w - 1);
        let y_prev = y.saturating_sub(1);
        let y_next = (y + 1).min(h - 1);

        let row_curr = y * w;
        let row_prev = y_prev * w;
        let row_next = y_next * w;

        // Central difference gradient
        let gx = (self.smoothed[row_curr + x_next] - self.smoothed[row_curr + x_prev]) * 0.5;
        let gy = (self.smoothed[row_next + x] - self.smoothed[row_prev + x]) * 0.5;

        let grad_sq = gx * gx + gy * gy;
        if grad_sq < 1e-6 {
            return (0.0, 0.0);
        }

        let grad_mag = grad_sq.sqrt();
        let depth_factor = (cfg.depth * 0.5).max(0.5);

        // Profile-dependent normal curvature
        let nz = match cfg.profile {
            1 => {
                // Chisel / Linear: Crisp planar angle
                0.6
            }
            2 => {
                // Ridge / Stepped: Multi-facet harmonic ripple
                let wave = (grad_mag * std::f32::consts::PI * 3.0).cos().abs();
                0.3 + 0.7 * wave
            }
            3 => {
                // Rim / Accent: Accentuate outer border transition
                let rim_shape = (-12.0 * (grad_mag - 0.4).powi(2)).exp();
                0.2 + 0.8 * (1.0 - rim_shape)
            }
            _ => {
                // Curved (Glossy)
                (1.0 - (grad_mag * depth_factor).clamp(0.0, 0.95).powi(2)).sqrt()
            }
        };

        // Surface normal N pointing towards observer (+z)
        let nx = -gx * depth_factor;
        let ny = -gy * depth_factor;
        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-5);
        let (nx, ny, nz) = (nx / len, ny / len, nz / len);

        // Directional light vector L
        let (lx, ly) = light_dir;
        let lz = 0.65f32;
        let l_len = (lx * lx + ly * ly + lz * lz).sqrt();
        let (lx, ly, lz) = (lx / l_len, ly / l_len, lz / l_len);

        // Diffuse N · L
        let ndotl = nx * lx + ny * ly + nz * lz;

        // Halfway specular vector H
        let (hx, hy, hz) = (lx, ly, lz + 1.0);
        let h_len = (hx * hx + hy * hy + hz * hz).sqrt();
        let (hx, hy, hz) = (hx / h_len, hy / h_len, hz / h_len);

        let ndoth = (nx * hx + ny * hy + nz * hz).max(0.0);
        // Glossy blinn-phong specular exponent
        let spec = ndoth.powi(16);

        let edge_hl = (grad_mag * 2.5).min(1.0)
            * (ndotl.max(0.0) * 0.4 + spec * 1.6)
            * (cfg.intensity / 100.0);

        let rim_shadow = if ndotl < 0.0 {
            (-ndotl * (grad_mag * 2.0).min(1.0) * (cfg.shadow_intensity / 100.0)).min(1.0)
        } else {
            0.0
        };

        (edge_hl, rim_shadow)
    }
}

/// Separable box blur for fast relief heightmap generation.
fn smooth_separable(src: &[f32], width: usize, height: usize, radius: usize) -> Vec<f32> {
    if radius == 0 || width == 0 || height == 0 {
        return src.to_vec();
    }

    let mut temp = vec![0.0f32; width * height];
    let mut out = vec![0.0f32; width * height];

    // Horizontal pass
    for y in 0..height {
        let row_offset = y * width;
        let mut sum = 0.0f32;
        let mut count = 0usize;

        // Initial window
        for x in 0..radius.min(width) {
            sum += src[row_offset + x];
            count += 1;
        }

        for x in 0..width {
            if x + radius < width {
                sum += src[row_offset + x + radius];
                count += 1;
            }
            if x > radius {
                sum -= src[row_offset + x - radius - 1];
                count -= 1;
            }
            temp[row_offset + x] = if count > 0 { sum / count as f32 } else { 0.0 };
        }
    }

    // Vertical pass
    for x in 0..width {
        let mut sum = 0.0f32;
        let mut count = 0usize;

        for y in 0..radius.min(height) {
            sum += temp[y * width + x];
            count += 1;
        }

        for y in 0..height {
            if y + radius < height {
                sum += temp[(y + radius) * width + x];
                count += 1;
            }
            if y > radius {
                sum -= temp[(y - radius - 1) * width + x];
                count -= 1;
            }
            out[y * width + x] = if count > 0 { sum / count as f32 } else { 0.0 };
        }
    }

    out
}
