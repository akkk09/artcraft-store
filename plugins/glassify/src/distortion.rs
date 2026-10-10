//! Procedural distortion sources for Glassify.
//!
//! Provides static and animated displacement/heightfield generators:
//! - Liquid Waves: Multi-octave sinusoidal undulations simulating fluid/molten glass.
//! - Frosted Grain: Micro-roughness hash stippling simulating etched or sandblasted glass.
//! - Caustics: Dynamic Voronoi cellular network simulating underwater caustic refraction.
//! - Ribbed / Fluted Glass: Cylindrical architectural grooves and reeded glass textures.
//!
//! All generators are deterministic, continuous, and fully procedural.

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistortionType {
    None = 0,
    LiquidWaves = 1,
    FrostedGrain = 2,
    Caustics = 3,
    RibbedFluted = 4,
}

impl DistortionType {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => DistortionType::LiquidWaves,
            2 => DistortionType::FrostedGrain,
            3 => DistortionType::Caustics,
            4 => DistortionType::RibbedFluted,
            _ => DistortionType::None,
        }
    }
}

/// Computes procedural distortion value at (x, y) at a given time.
/// Returns value roughly in [-1.0, 1.0].
pub fn sample_distortion(
    x: f32,
    y: f32,
    time: f32,
    dist_type: DistortionType,
    scale: f32,
    amount: f32,
    speed: f32,
) -> f32 {
    if dist_type == DistortionType::None || amount <= 0.001 || scale <= 0.001 {
        return 0.0;
    }

    let t = time * speed;
    let inv_s = 1.0 / scale.max(1.0);
    let u = x * inv_s;
    let v = y * inv_s;

    let raw = match dist_type {
        DistortionType::None => 0.0,
        DistortionType::LiquidWaves => sample_liquid_waves(u, v, t),
        DistortionType::FrostedGrain => sample_frosted_grain(u, v),
        DistortionType::Caustics => sample_caustics(u, v, t),
        DistortionType::RibbedFluted => sample_ribbed_fluted(u, v),
    };

    raw * (amount / 100.0)
}

/// Evaluates finite-difference gradient of distortion field (d/dx, d/dy).
pub fn sample_distortion_gradient(
    x: f32,
    y: f32,
    time: f32,
    dist_type: DistortionType,
    scale: f32,
    amount: f32,
    speed: f32,
) -> (f32, f32) {
    if dist_type == DistortionType::None || amount <= 0.001 || scale <= 0.001 {
        return (0.0, 0.0);
    }

    let eps = (scale * 0.05).clamp(0.5, 2.0);
    let v_x_pos = sample_distortion(x + eps, y, time, dist_type, scale, amount, speed);
    let v_x_neg = sample_distortion(x - eps, y, time, dist_type, scale, amount, speed);
    let v_y_pos = sample_distortion(x, y + eps, time, dist_type, scale, amount, speed);
    let v_y_neg = sample_distortion(x, y - eps, time, dist_type, scale, amount, speed);

    let inv_2eps = 1.0 / (2.0 * eps);
    ((v_x_pos - v_x_neg) * inv_2eps, (v_y_pos - v_y_neg) * inv_2eps)
}

/// Multi-octave sinusoidal liquid waves.
fn sample_liquid_waves(u: f32, v: f32, t: f32) -> f32 {
    let w1 = ((u * 1.8 + v * 0.9 + t * 1.5).sin() * (v * 1.4 - u * 0.7 - t * 1.1).cos()) * 0.5;
    let w2 = ((u * 3.7 - v * 2.1 + t * 2.3).sin() + (u * 1.2 + v * 3.4 - t * 1.8).cos()) * 0.25;
    let w3 = ((u * 7.5 + v * 6.3 + t * 3.1).sin()) * 0.15;
    let w4 = ((u * 12.0 - v * 10.0 + t * 4.2).sin()) * 0.1;
    (w1 + w2 + w3 + w4).clamp(-1.0, 1.0)
}

/// Pseudo-random hash for frosted / etched micro-grain.
fn sample_frosted_grain(u: f32, v: f32) -> f32 {
    // 2D Hash function producing high frequency micro-stipple
    let n = (u * 12.9898 + v * 78.233).sin() * 43758.547;
    let frac = n - n.floor();
    (frac * 2.0 - 1.0).clamp(-1.0, 1.0)
}

/// Cellular Voronoi caustics simulation with animated feature points.
fn sample_caustics(u: f32, v: f32, t: f32) -> f32 {
    let cu = u.floor();
    let cv = v.floor();
    let fu = u - cu;
    let fv = v - cv;

    let mut d1 = 10.0f32;
    let mut d2 = 10.0f32;

    for j in -1..=1 {
        for i in -1..=1 {
            let gi = i as f32;
            let gj = j as f32;
            let cell_x = cu + gi;
            let cell_y = cv + gj;

            // Deterministic hash point in cell
            let h1 = ((cell_x * 127.1 + cell_y * 311.7).sin() * 43758.545).fract();
            let h2 = ((cell_x * 269.5 + cell_y * 183.3).sin() * 43758.545).fract();

            // Animate feature point within the cell
            let px = gi + 0.5 + 0.35 * (t * 1.2 + h1 * 2.0 * PI).sin();
            let py = gj + 0.5 + 0.35 * (t * 1.4 + h2 * 2.0 * PI).cos();

            let dx = px - fu;
            let dy = py - fv;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < d1 {
                d2 = d1;
                d1 = dist;
            } else if dist < d2 {
                d2 = dist;
            }
        }
    }

    // Classic caustic web filament equation: (d2 - d1)
    let caustic = (d2 - d1).clamp(0.0, 1.0);
    // Expand to [-1.0, 1.0] with contrast
    let curved = (caustic * 2.5).clamp(0.0, 1.0);
    curved * 2.0 - 1.0
}

/// Cylindrical architectural fluted glass with curved periodic ridges.
fn sample_ribbed_fluted(u: f32, _v: f32) -> f32 {
    // Semicircular cylindrical flute cross-section
    let period = u - u.floor(); // in [0, 1)
    let centered = (period - 0.5) * 2.0; // in [-1, 1]
    let circular_profile = (1.0 - centered * centered).max(0.0).sqrt();
    circular_profile * 2.0 - 1.0
}
