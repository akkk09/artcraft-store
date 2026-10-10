//! Sweep geometry, falloff profiles, chromatic dispersion, and time-based animation for LumaSweep.

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[derive(Debug, Clone, Copy)]
pub struct BeamProfile {
    pub shape: usize,
    pub width: f32,
    pub softness: f32,
    pub chromatic_fringe: f32,
    pub glow_intensity: f32,
    pub glow_radius: f32,
}

impl BeamProfile {
    /// Calculate base beam falloff for a given distance to the sweep axis `d` (in pixels).
    #[inline]
    pub fn evaluate_falloff(&self, d: f32) -> f32 {
        let half = (self.width * 0.5).max(0.5);
        let soft_factor = (0.25 + 0.75 * (self.softness / 50.0)).clamp(0.1, 4.0);
        let effective_half = half * soft_factor;
        let xi = (d.abs() / effective_half).max(0.0);

        match self.shape {
            0 => {
                // Smooth Hermite
                smoothstep(1.0, 0.0, xi)
            }
            1 => {
                // Linear
                (1.0 - xi).max(0.0)
            }
            2 => {
                // Sharp Peak
                if xi >= 1.0 {
                    0.0
                } else {
                    let root = (1.0 - xi.sqrt()).max(0.0);
                    root * root
                }
            }
            3 => {
                // Gaussian
                (-3.0 * xi * xi).exp()
            }
            4 => {
                // Asymmetric Leaning Flare
                if d >= 0.0 {
                    smoothstep(1.0, 0.0, d / effective_half)
                } else {
                    smoothstep(1.0, 0.0, -d / (effective_half * 2.2))
                }
            }
            _ => smoothstep(1.0, 0.0, xi),
        }
    }

    /// Evaluate chromatic dispersion across R, G, B channels.
    ///
    /// When `chromatic_fringe > 0`, spectral components diverge spatially across the wavefront.
    #[inline]
    pub fn evaluate_chromatic(&self, d: f32) -> [f32; 3] {
        if self.chromatic_fringe <= 0.001 {
            let b = self.evaluate_falloff(d);
            return [b, b, b];
        }

        let half = (self.width * 0.5).max(0.5);
        let offset = half * (self.chromatic_fringe / 100.0) * 0.25;

        let r = self.evaluate_falloff(d - offset);
        let g = self.evaluate_falloff(d);
        let b = self.evaluate_falloff(d + offset);

        [r, g, b]
    }

    /// Evaluate secondary diffused glow halo.
    #[inline]
    pub fn evaluate_glow(&self, d: f32) -> f32 {
        if self.glow_intensity <= 0.001 {
            return 0.0;
        }
        let half = (self.width * 0.5).max(0.5);
        let glow_w = half + self.glow_radius.max(1.0);
        let xi = d / glow_w;
        let g = (-2.0 * xi * xi).exp();
        g * (self.glow_intensity / 100.0)
    }
}

/// Compute animated center position if auto-animation is enabled.
pub fn compute_center(
    manual_center: (f64, f64),
    width: usize,
    height: usize,
    dir_rad: f64,
    auto_animate: bool,
    speed: f64,
    loop_mode: usize,
    phase_deg: f64,
    time: f64,
) -> (f64, f64) {
    if !auto_animate {
        return manual_center;
    }

    let w = width as f64;
    let h = height as f64;
    let diag = (w * w + h * h).sqrt();
    let extent = diag * 0.65;

    let nx = dir_rad.cos();
    let ny = dir_rad.sin();

    let phase_norm = phase_deg / 360.0;
    let progress = time * speed + phase_norm;

    let t_norm = match loop_mode {
        1 => {
            // Ping-Pong (Triangle wave between -1 and 1)
            let cycle = progress - (progress + 0.5).floor();
            cycle.abs() * 4.0 - 1.0
        }
        _ => {
            // One-Way Repeat (Sawtooth wave from -1 to 1)
            let fract = progress - progress.floor();
            fract * 2.0 - 1.0
        }
    };

    let cx = w * 0.5 + nx * (t_norm * extent);
    let cy = h * 0.5 + ny * (t_norm * extent);

    (cx, cy)
}
