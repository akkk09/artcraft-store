//! EaseCraft Core Engine: mathematical solvers, boundary validations, and preset management
//! for EffectCraft timeline keyframe easing.

use serde::{Deserialize, Serialize};

pub const MIN_INFLUENCE: f64 = 0.1;
pub const MAX_INFLUENCE: f64 = 100.0;
pub const MAX_SPEED: f64 = 100.0;
pub const EPSILON: f64 = 1e-9;

/// A 1D cubic Bézier easing curve represented in After Effects / EffectCraft keyframe terms:
/// Out/In influences (in percent, 0.1% to 100%) and speeds relative to the segment's average
/// speed (1.0 = linear slope, 0.0 = at rest, negative = overshoot / anticipation).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EaseCurve {
    pub out_influence: f64,
    pub out_speed: f64,
    pub in_influence: f64,
    pub in_speed: f64,
}

impl Default for EaseCurve {
    fn default() -> Self {
        Self::linear()
    }
}

impl EaseCurve {
    pub fn new(out_inf: f64, out_spd: f64, in_inf: f64, in_spd: f64) -> Result<Self, String> {
        if !out_inf.is_finite() || !out_spd.is_finite() || !in_inf.is_finite() || !in_spd.is_finite() {
            return Err("Curve values must be finite numbers".to_string());
        }
        let out_influence = out_inf.clamp(MIN_INFLUENCE, MAX_INFLUENCE);
        let in_influence = in_inf.clamp(MIN_INFLUENCE, MAX_INFLUENCE);
        let out_speed = out_spd.clamp(-MAX_SPEED, MAX_SPEED);
        let in_speed = in_spd.clamp(-MAX_SPEED, MAX_SPEED);
        Ok(Self { out_influence, out_speed, in_influence, in_speed })
    }

    /// Evaluates 1D cubic Bézier basis at parameter `u` in [0, 1].
    #[inline]
    pub fn bez(p0: f64, p1: f64, p2: f64, p3: f64, u: f64) -> f64 {
        let v = 1.0 - u;
        v * v * v * p0 + 3.0 * v * v * u * p1 + 3.0 * v * u * u * p2 + u * u * u * p3
    }

    /// Solves the parameter `u` in [0, 1] for a given time progress `x` in [0, 1] using
    /// Newton-Raphson iterations with bisection fallback.
    pub fn solve_u(&self, x: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        let (x1, x2) = (self.out_influence / 100.0, 1.0 - self.in_influence / 100.0);
        let (p0, p1, p2, p3) = (0.0, x1, x2, 1.0);

        // Initial guess
        let mut u = x;
        for _ in 0..12 {
            let f = Self::bez(p0, p1, p2, p3, u) - x;
            if f.abs() < 1e-7 {
                return u;
            }
            // First derivative B'(u)
            let v = 1.0 - u;
            let d = 3.0 * v * v * (p1 - p0) + 6.0 * v * u * (p2 - p1) + 3.0 * u * u * (p3 - p2);
            if d.abs() < 1e-6 {
                break;
            }
            let next = u - f / d;
            if !(0.0..=1.0).contains(&next) {
                break;
            }
            u = next;
        }

        // Bisection fallback
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..60 {
            let mid = (lo + hi) * 0.5;
            if Self::bez(p0, p1, p2, p3, mid) < x {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (lo + hi) * 0.5
    }

    /// Evaluates the normalized progress `y` in R at normalized time `x` in [0, 1].
    pub fn eval(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }
        let u = self.solve_u(x);
        let y0 = 0.0;
        let y1 = (self.out_influence / 100.0) * self.out_speed;
        let y2 = 1.0 - (self.in_influence / 100.0) * self.in_speed;
        let y3 = 1.0;
        Self::bez(y0, y1, y2, y3, u)
    }

    /// Mirrors the curve horizontally by swapping Out and In controls.
    pub fn mirror(&self) -> Self {
        Self {
            out_influence: self.in_influence,
            out_speed: self.in_speed,
            in_influence: self.out_influence,
            in_speed: self.out_speed,
        }
    }

    /// Reverses the curve trajectory progression.
    pub fn reverse(&self) -> Self {
        Self {
            out_influence: self.in_influence,
            out_speed: 2.0 - self.in_speed,
            in_influence: self.out_influence,
            in_speed: 2.0 - self.out_speed,
        }
    }

    // --- Standard & Mathematical Presets ---

    pub fn linear() -> Self {
        Self { out_influence: 100.0 / 3.0, out_speed: 1.0, in_influence: 100.0 / 3.0, in_speed: 1.0 }
    }

    pub fn ease_in() -> Self {
        Self { out_influence: 33.333, out_speed: 0.0, in_influence: 33.333, in_speed: 1.0 }
    }

    pub fn ease_out() -> Self {
        Self { out_influence: 33.333, out_speed: 1.0, in_influence: 33.333, in_speed: 0.0 }
    }

    pub fn ease_in_out() -> Self {
        Self { out_influence: 50.0, out_speed: 0.0, in_influence: 50.0, in_speed: 0.0 }
    }

    pub fn ease_in_out_soft() -> Self {
        Self { out_influence: 25.0, out_speed: 0.0, in_influence: 25.0, in_speed: 0.0 }
    }

    pub fn ease_in_out_strong() -> Self {
        Self { out_influence: 75.0, out_speed: 0.0, in_influence: 75.0, in_speed: 0.0 }
    }

    pub fn ease_in_out_extreme() -> Self {
        Self { out_influence: 88.0, out_speed: 0.0, in_influence: 88.0, in_speed: 0.0 }
    }

    pub fn back_in() -> Self {
        // Anticipation: dips backward before accelerating
        Self { out_influence: 45.0, out_speed: -0.85, in_influence: 33.333, in_speed: 1.2 }
    }

    pub fn back_out() -> Self {
        // Overshoot: shoots past destination before settling
        Self { out_influence: 33.333, out_speed: 1.2, in_influence: 45.0, in_speed: -0.85 }
    }

    pub fn back_in_out() -> Self {
        // Symmetrical anticipation and overshoot
        Self { out_influence: 50.0, out_speed: -0.75, in_influence: 50.0, in_speed: -0.75 }
    }

    pub fn elastic_snap() -> Self {
        // Builds tension, then snaps through
        Self { out_influence: 70.0, out_speed: 0.0, in_influence: 20.0, in_speed: -1.5 }
    }

    pub fn bounce_settle() -> Self {
        // High initial speed with damped settle
        Self { out_influence: 35.0, out_speed: 1.8, in_influence: 65.0, in_speed: -0.4 }
    }

    pub fn smooth() -> Self {
        Self { out_influence: 40.0, out_speed: 0.25, in_influence: 40.0, in_speed: 0.25 }
    }
}

/// A named preset with category metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub category: String,
    pub curve: EaseCurve,
}

/// Open JSON preset collection container.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresetPack {
    pub format: String,
    pub version: u32,
    pub presets: Vec<Preset>,
}

impl Default for PresetPack {
    fn default() -> Self {
        Self {
            format: "easecraft-presets".to_string(),
            version: 1,
            presets: vec![
                Preset { name: "Linear".into(), category: "Basic".into(), curve: EaseCurve::linear() },
                Preset { name: "Ease In".into(), category: "Basic".into(), curve: EaseCurve::ease_in() },
                Preset { name: "Ease Out".into(), category: "Basic".into(), curve: EaseCurve::ease_out() },
                Preset { name: "Ease In-Out Soft".into(), category: "Standard".into(), curve: EaseCurve::ease_in_out_soft() },
                Preset { name: "Ease In-Out".into(), category: "Standard".into(), curve: EaseCurve::ease_in_out() },
                Preset { name: "Ease In-Out Strong".into(), category: "Standard".into(), curve: EaseCurve::ease_in_out_strong() },
                Preset { name: "Ease In-Out Extreme".into(), category: "Standard".into(), curve: EaseCurve::ease_in_out_extreme() },
                Preset { name: "Back In (Anticipate)".into(), category: "Dynamic".into(), curve: EaseCurve::back_in() },
                Preset { name: "Back Out (Overshoot)".into(), category: "Dynamic".into(), curve: EaseCurve::back_out() },
                Preset { name: "Back In-Out".into(), category: "Dynamic".into(), curve: EaseCurve::back_in_out() },
                Preset { name: "Elastic Snap".into(), category: "Dynamic".into(), curve: EaseCurve::elastic_snap() },
                Preset { name: "Bounce Settle".into(), category: "Dynamic".into(), curve: EaseCurve::bounce_settle() },
                Preset { name: "Smooth S-Curve".into(), category: "Standard".into(), curve: EaseCurve::smooth() },
            ],
        }
    }
}

impl PresetPack {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialization error: {e}"))
    }

    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let pack: Self = serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON format: {e}"))?;
        if pack.format != "easecraft-presets" {
            return Err("Unknown format identifier".to_string());
        }
        Ok(pack)
    }
}

/// Computes the 3D Bézier motion path arc length using adaptive segment integration.
pub fn path_length_3d(a: [f64; 3], out_tan: [f64; 3], in_tan: [f64; 3], b: [f64; 3]) -> f64 {
    let p0 = a;
    let p1 = [a[0] + out_tan[0], a[1] + out_tan[1], a[2] + out_tan[2]];
    let p2 = [b[0] + in_tan[0], b[1] + in_tan[1], b[2] + in_tan[2]];
    let p3 = b;

    let mut total_len = 0.0;
    let mut prev = p0;
    const STEPS: usize = 64;
    for s in 1..=STEPS {
        let t = s as f64 / STEPS as f64;
        let pt = [
            EaseCurve::bez(p0[0], p1[0], p2[0], p3[0], t),
            EaseCurve::bez(p0[1], p1[1], p2[1], p3[1], t),
            EaseCurve::bez(p0[2], p1[2], p2[2], p3[2], t),
        ];
        let d = ((pt[0] - prev[0]).powi(2) + (pt[1] - prev[1]).powi(2) + (pt[2] - prev[2]).powi(2)).sqrt();
        total_len += d;
        prev = pt;
    }
    total_len
}
