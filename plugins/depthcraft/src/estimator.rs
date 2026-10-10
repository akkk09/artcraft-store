//! Embedded zero-weights monocular depth estimator.
//!
//! Uses multi-scale structure gradients, atmospheric haze cues, and perspective priors
//! to synthesize an initial continuous depth map without requiring external neural network weights.

pub fn estimate_depth_proxy(rgb: &[[f32; 3]], width: usize, height: usize, out: &mut [f32]) {
    if width == 0 || height == 0 || rgb.len() < width * height || out.len() < width * height {
        return;
    }

    // Step 1: Compute luminance and saturation per pixel
    let mut lum = vec![0.0f32; width * height];
    let mut sat = vec![0.0f32; width * height];
    for (i, c) in rgb.iter().take(width * height).enumerate() {
        let (r, g, b) = (c[0], c[1], c[2]);
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        let s = if max_c > 1e-4 { (max_c - min_c) / max_c } else { 0.0 };

        if let Some(target) = lum.get_mut(i) {
            *target = y;
        }
        if let Some(target) = sat.get_mut(i) {
            *target = s;
        }
    }

    // Step 2: Multi-scale gradient magnitude (local sharpness/focus cue)
    let inv_h = 1.0 / (height.max(1) as f32);

    for y in 0..height {
        let row_start = y * width;
        let y_prev = if y > 0 { y - 1 } else { 0 };
        let y_next = if y + 1 < height { y + 1 } else { y };
        let row_prev = y_prev * width;
        let row_next = y_next * width;

        // Perspective prior: lower in frame is generally nearer (0.0 = near, 1.0 = far)
        // Normalized vertical position: 0.0 at bottom, 1.0 at top
        let vert_prior = 1.0 - ((y as f32) * inv_h);

        for x in 0..width {
            let idx = row_start + x;
            let x_prev = if x > 0 { x - 1 } else { 0 };
            let x_next = if x + 1 < width { x + 1 } else { x };

            let l_c = lum.get(idx).copied().unwrap_or(0.0);
            let s_c = sat.get(idx).copied().unwrap_or(0.0);

            let l_l = lum.get(row_start + x_prev).copied().unwrap_or(l_c);
            let l_r = lum.get(row_start + x_next).copied().unwrap_or(l_c);
            let l_u = lum.get(row_prev + x).copied().unwrap_or(l_c);
            let l_d = lum.get(row_next + x).copied().unwrap_or(l_c);

            // Sobel / central difference gradient
            let dx = (l_r - l_l) * 0.5;
            let dy = (l_d - l_u) * 0.5;
            let grad = (dx * dx + dy * dy).sqrt();

            // Atmospheric perspective / haze prior:
            // Distant objects lose saturation and shift towards atmospheric luminance
            let haze_cue = (1.0 - s_c) * l_c;

            // Sharp high-contrast edges cue: in-focus foreground has high sharpness
            let sharpness_cue = (1.0 - (grad * 2.5).clamp(0.0, 1.0)).clamp(0.0, 1.0);

            // Weighted combination of cues
            let raw_depth = 0.55 * vert_prior + 0.25 * haze_cue + 0.20 * sharpness_cue;

            if let Some(target) = out.get_mut(idx) {
                *target = raw_depth.clamp(0.0, 1.0);
            }
        }
    }
}
