//! Procedural surface micro-texture generators for LumaSweep.

#[inline(always)]
fn hash_int(mut n: u32) -> f32 {
    n = ((n >> 16) ^ n).wrapping_mul(0x45d9f3b);
    n = ((n >> 16) ^ n).wrapping_mul(0x45d9f3b);
    n = (n >> 16) ^ n;
    (n & 0x007fffff) as f32 / 8388607.0
}

#[inline(always)]
fn hash2d(x: i32, y: i32) -> f32 {
    let n = (x as u32).wrapping_mul(0x119de1f3) ^ (y as u32).wrapping_mul(0x45d9f3b);
    hash_int(n)
}

/// Compute procedural surface roughness modifier in range [-1.0, 1.0].
///
/// * `d`: Perpendicular distance to the sweep axis (in buffer pixels).
/// * `s`: Parallel coordinate along the sweep tangent (in buffer pixels).
/// * `mode`: 0 = Brushed Anisotropic, 1 = Micro Grain, 2 = Satin Cross.
/// * `scale`: Texture scale parameter in pixels.
#[inline]
pub fn sample_surface_texture(d: f64, s: f64, mode: usize, scale: f32) -> f32 {
    let sc = scale.max(1.0) as f64;
    match mode {
        0 => {
            // Brushed Anisotropic: Tight striations across d with elongated grain along s
            let d_quant = (d / (sc * 0.25)).floor() as i32;
            let s_quant = (s / (sc * 4.0)).floor() as i32;
            let h1 = hash2d(d_quant, s_quant);
            let h2 = hash2d(d_quant * 3 + 17, (s / (sc * 1.5)).floor() as i32);
            let fine = (d / (sc * 0.125) * std::f64::consts::PI).sin() as f32;
            ((h1 * 0.6 + h2 * 0.4 + fine * 0.2) - 0.5) * 2.0
        }
        1 => {
            // Fine Micro Grain: 2D isotropic stipple
            let qx = (d / (sc * 0.5)).floor() as i32;
            let qy = (s / (sc * 0.5)).floor() as i32;
            let h = hash2d(qx, qy);
            (h - 0.5) * 2.0
        }
        2 => {
            // Satin Cross: Smooth harmonic woven texture
            let u = (d / sc * std::f64::consts::PI * 2.0) as f32;
            let v = (s / sc * std::f64::consts::PI * 2.0) as f32;
            let wave = (u.sin() * v.cos() + (u * 2.0).cos() * (v * 2.0).sin() * 0.5) * 0.66;
            wave.clamp(-1.0, 1.0)
        }
        _ => 0.0,
    }
}
