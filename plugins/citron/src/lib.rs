//! Citron Core Curve Engine: Mathematical solvers, multi-channel curve modeling,
//! Free-Form Deformation (FFD) lattice cage manipulation, and de Casteljau subdivision
//! for EffectCraft's professional graph editing workspace.
//!
//! Provides:
//! - Multi-channel curve data structures with color-coding and visibility states.
//! - High-precision cubic Bézier evaluation and inversion via Newton-Raphson.
//! - De Casteljau subdivision for inserting keyframes without trajectory degradation.
//! - Free-Form Deformation (FFD) 2D lattice cage retiming, scaling, and skewing.
//! - Multi-channel view normalization (Normalized 0-1, Stacked lanes, Absolute, Speed).
//! - Buffer curve snapshotting and RMSE before/after difference analysis.
//! - Broken vs. Unified tangent handle alignment and clamping algorithms.

use serde::{Deserialize, Serialize};

pub const MIN_INFLUENCE: f64 = 0.1;
pub const MAX_INFLUENCE: f64 = 100.0;
pub const MAX_SPEED: f64 = 1000.0;
pub const EPSILON: f64 = 1e-9;

/// Keyframe temporal interpolation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InterpType {
    #[default]
    Bezier,
    Linear,
    Hold,
}

/// Tangent handle parameters defining velocity and influence entering or leaving a keyframe.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TangentHandle {
    /// Temporal influence in percent [0.1%, 100.0%].
    pub influence: f64,
    /// Tangent speed / slope (value units per second).
    pub speed: f64,
}

impl Default for TangentHandle {
    fn default() -> Self {
        Self {
            influence: 33.333,
            speed: 0.0,
        }
    }
}

impl TangentHandle {
    pub fn new(influence: f64, speed: f64) -> Self {
        Self {
            influence: influence.clamp(MIN_INFLUENCE, MAX_INFLUENCE),
            speed: speed.clamp(-MAX_SPEED, MAX_SPEED),
        }
    }
}

/// Single animated keyframe point in time and value space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyPoint {
    /// Time in seconds.
    pub time: f64,
    /// Value at this keyframe.
    pub value: f64,
    /// Incoming tangent handle (from preceding keyframe).
    pub in_tangent: TangentHandle,
    /// Outgoing tangent handle (towards following keyframe).
    pub out_tangent: TangentHandle,
    /// Interpolation type.
    pub interp: InterpType,
    /// Whether tangent handles are broken (independent angles) or unified (collinear).
    pub broken: bool,
    /// Selection state in the UI.
    pub selected: bool,
}

impl KeyPoint {
    pub fn new(time: f64, value: f64) -> Self {
        Self {
            time,
            value,
            in_tangent: TangentHandle::default(),
            out_tangent: TangentHandle::default(),
            interp: InterpType::Bezier,
            broken: false,
            selected: false,
        }
    }

    pub fn with_ease(time: f64, value: f64, in_inf: f64, in_spd: f64, out_inf: f64, out_spd: f64) -> Self {
        Self {
            time,
            value,
            in_tangent: TangentHandle::new(in_inf, in_spd),
            out_tangent: TangentHandle::new(out_inf, out_spd),
            interp: InterpType::Bezier,
            broken: false,
            selected: false,
        }
    }
}

/// An animated channel (e.g. "Position X", "Rotation", "Opacity").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelCurve {
    pub id: String,
    pub name: String,
    /// Display color in RGBA format [r, g, b, a].
    pub color: [f32; 4],
    pub visible: bool,
    pub locked: bool,
    pub keys: Vec<KeyPoint>,
}

impl ChannelCurve {
    pub fn new(id: impl Into<String>, name: impl Into<String>, color: [f32; 4]) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            color,
            visible: true,
            locked: false,
            keys: Vec::new(),
        }
    }

    /// Sorts keyframes chronologically by time.
    pub fn sort_keys(&mut self) {
        self.keys.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Computes bounding rectangle [time_min, time_max, val_min, val_max].
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        if self.keys.is_empty() {
            return (0.0, 1.0, 0.0, 1.0);
        }
        let mut t_min = self.keys[0].time;
        let mut t_max = self.keys[0].time;
        let mut v_min = self.keys[0].value;
        let mut v_max = self.keys[0].value;

        for k in &self.keys {
            t_min = t_min.min(k.time);
            t_max = t_max.max(k.time);
            v_min = v_min.min(k.value);
            v_max = v_max.max(k.value);
        }

        if (t_max - t_min).abs() < EPSILON {
            t_max += 1.0;
        }
        if (v_max - v_min).abs() < EPSILON {
            v_max += 1.0;
        }

        (t_min, t_max, v_min, v_max)
    }

    /// Evaluates value of the channel at time `t`.
    pub fn eval(&self, t: f64) -> f64 {
        if self.keys.is_empty() {
            return 0.0;
        }
        if self.keys.len() == 1 || t <= self.keys[0].time {
            return self.keys[0].value;
        }
        if t >= self.keys.last().unwrap().time {
            return self.keys.last().unwrap().value;
        }

        // Find surrounding keyframe segment
        for i in 0..(self.keys.len() - 1) {
            let k0 = &self.keys[i];
            let k1 = &self.keys[i + 1];
            if t >= k0.time && t <= k1.time {
                return eval_segment(k0, k1, t);
            }
        }

        self.keys.last().unwrap().value
    }

    /// Evaluates rate of change (derivative / speed) at time `t`.
    pub fn speed_at(&self, t: f64) -> f64 {
        let dt = 0.001;
        let v0 = self.eval(t - dt * 0.5);
        let v1 = self.eval(t + dt * 0.5);
        (v1 - v0) / dt
    }

    /// Inserts a new keyframe at time `t`, splitting the existing Bézier curve via
    /// de Casteljau's algorithm and preserving the exact geometric curve path.
    pub fn insert_key_at(&mut self, t: f64) -> Result<usize, String> {
        self.sort_keys();
        if self.keys.is_empty() {
            self.keys.push(KeyPoint::new(t, 0.0));
            return Ok(0);
        }

        if t < self.keys[0].time {
            let v = self.keys[0].value;
            self.keys.insert(0, KeyPoint::new(t, v));
            return Ok(0);
        }
        if t > self.keys.last().unwrap().time {
            let v = self.keys.last().unwrap().value;
            self.keys.push(KeyPoint::new(t, v));
            return Ok(self.keys.len() - 1);
        }

        // Check if key already exists very close to t
        for (idx, k) in self.keys.iter().enumerate() {
            if (k.time - t).abs() < 1e-4 {
                return Ok(idx);
            }
        }

        // Find containing segment
        let mut seg_idx = None;
        for i in 0..(self.keys.len() - 1) {
            if t > self.keys[i].time && t < self.keys[i + 1].time {
                seg_idx = Some(i);
                break;
            }
        }

        let Some(i) = seg_idx else {
            return Err("Time out of range".to_string());
        };

        let k0 = self.keys[i].clone();
        let k1 = self.keys[i + 1].clone();

        let (left_sub, right_sub, new_key) = split_bezier_segment(&k0, &k1, t);

        // Update k0 and k1 tangents, and insert new key
        self.keys[i].out_tangent = left_sub;
        self.keys[i + 1].in_tangent = right_sub;
        self.keys.insert(i + 1, new_key);

        Ok(i + 1)
    }
}

/// Evaluates 1D cubic Bézier basis `B(u) = (1-u)^3 P0 + 3(1-u)^2 u P1 + 3(1-u) u^2 P2 + u^3 P3`.
#[inline]
pub fn bez1d(p0: f64, p1: f64, p2: f64, p3: f64, u: f64) -> f64 {
    let v = 1.0 - u;
    v * v * v * p0 + 3.0 * v * v * u * p1 + 3.0 * v * u * u * p2 + u * u * u * p3
}

/// Derivative `B'(u)`.
#[inline]
pub fn bez1d_deriv(p0: f64, p1: f64, p2: f64, p3: f64, u: f64) -> f64 {
    let v = 1.0 - u;
    3.0 * v * v * (p1 - p0) + 6.0 * v * u * (p2 - p1) + 3.0 * u * u * (p3 - p2)
}

/// Solves parameter `u` in [0, 1] for time `t` between `k0.time` and `k1.time`.
pub fn solve_segment_u(k0: &KeyPoint, k1: &KeyPoint, t: f64) -> f64 {
    let dt = (k1.time - k0.time).max(EPSILON);
    let x_target = ((t - k0.time) / dt).clamp(0.0, 1.0);

    let p0 = 0.0;
    let p1 = k0.out_tangent.influence / 100.0;
    let p2 = 1.0 - k1.in_tangent.influence / 100.0;
    let p3 = 1.0;

    // Newton-Raphson with bisection fallback
    let mut u = x_target;
    for _ in 0..16 {
        let f = bez1d(p0, p1, p2, p3, u) - x_target;
        if f.abs() < 1e-7 {
            return u;
        }
        let d = bez1d_deriv(p0, p1, p2, p3, u);
        if d.abs() < 1e-6 {
            break;
        }
        let next = u - f / d;
        if !(0.0..=1.0).contains(&next) {
            break;
        }
        u = next;
    }

    let mut lo = 0.0;
    let mut hi = 1.0;
    for _ in 0..48 {
        let mid = (lo + hi) * 0.5;
        if bez1d(p0, p1, p2, p3, mid) < x_target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) * 0.5
}

/// Evaluates value on segment between `k0` and `k1` at time `t`.
pub fn eval_segment(k0: &KeyPoint, k1: &KeyPoint, t: f64) -> f64 {
    match k0.interp {
        InterpType::Hold => k0.value,
        InterpType::Linear => {
            let dt = (k1.time - k0.time).max(EPSILON);
            let frac = ((t - k0.time) / dt).clamp(0.0, 1.0);
            k0.value + frac * (k1.value - k0.value)
        }
        InterpType::Bezier => {
            let dt = (k1.time - k0.time).max(EPSILON);
            let u = solve_segment_u(k0, k1, t);

            let v0 = k0.value;
            let v1 = k0.value + (k0.out_tangent.influence / 100.0) * k0.out_tangent.speed * dt;
            let v2 = k1.value - (k1.in_tangent.influence / 100.0) * k1.in_tangent.speed * dt;
            let v3 = k1.value;

            bez1d(v0, v1, v2, v3, u)
        }
    }
}

/// De Casteljau subdivision: splits segment `[k0, k1]` at time `t` into two cubic Bézier segments
/// returning `(k0_new_out_tangent, k1_new_in_tangent, new_key_point)`.
pub fn split_bezier_segment(
    k0: &KeyPoint,
    k1: &KeyPoint,
    t: f64,
) -> (TangentHandle, TangentHandle, KeyPoint) {
    let dt = (k1.time - k0.time).max(EPSILON);
    let u = solve_segment_u(k0, k1, t);

    // 2D control points P = (time, value)
    let p0 = (k0.time, k0.value);
    let p1 = (
        k0.time + dt * (k0.out_tangent.influence / 100.0),
        k0.value + dt * (k0.out_tangent.influence / 100.0) * k0.out_tangent.speed,
    );
    let p2 = (
        k1.time - dt * (k1.in_tangent.influence / 100.0),
        k1.value - dt * (k1.in_tangent.influence / 100.0) * k1.in_tangent.speed,
    );
    let p3 = (k1.time, k1.value);

    // Level 1 de Casteljau points
    let lerp2d = |a: (f64, f64), b: (f64, f64), s: f64| -> (f64, f64) {
        (a.0 + s * (b.0 - a.0), a.1 + s * (b.1 - a.1))
    };

    let q0 = lerp2d(p0, p1, u);
    let q1 = lerp2d(p1, p2, u);
    let q2 = lerp2d(p2, p3, u);

    // Level 2 points
    let r0 = lerp2d(q0, q1, u);
    let r1 = lerp2d(q1, q2, u);

    // Split split-point P(u)
    let s = lerp2d(r0, r1, u);

    let dt_left = (s.0 - p0.0).max(EPSILON);
    let dt_right = (p3.0 - s.0).max(EPSILON);

    // Left segment: p0 -> q0 -> r0 -> s
    let out_inf_left = (((q0.0 - p0.0) / dt_left) * 100.0).clamp(MIN_INFLUENCE, MAX_INFLUENCE);
    let out_spd_left = if (q0.0 - p0.0).abs() > EPSILON {
        (q0.1 - p0.1) / (q0.0 - p0.0)
    } else {
        k0.out_tangent.speed
    };

    let in_inf_split = (((s.0 - r0.0) / dt_left) * 100.0).clamp(MIN_INFLUENCE, MAX_INFLUENCE);
    let in_spd_split = if (s.0 - r0.0).abs() > EPSILON {
        (s.1 - r0.1) / (s.0 - r0.0)
    } else {
        0.0
    };

    // Right segment: s -> r1 -> q2 -> p3
    let out_inf_split = (((r1.0 - s.0) / dt_right) * 100.0).clamp(MIN_INFLUENCE, MAX_INFLUENCE);
    let out_spd_split = if (r1.0 - s.0).abs() > EPSILON {
        (r1.1 - s.1) / (r1.0 - s.0)
    } else {
        0.0
    };

    let in_inf_right = (((p3.0 - q2.0) / dt_right) * 100.0).clamp(MIN_INFLUENCE, MAX_INFLUENCE);
    let in_spd_right = if (p3.0 - q2.0).abs() > EPSILON {
        (p3.1 - q2.1) / (p3.0 - q2.0)
    } else {
        k1.in_tangent.speed
    };

    let k0_out = TangentHandle::new(out_inf_left, out_spd_left);
    let k1_in = TangentHandle::new(in_inf_right, in_spd_right);

    let new_key = KeyPoint {
        time: s.0,
        value: s.1,
        in_tangent: TangentHandle::new(in_inf_split, in_spd_split),
        out_tangent: TangentHandle::new(out_inf_split, out_spd_split),
        interp: k0.interp,
        broken: false,
        selected: true,
    };

    (k0_out, k1_in, new_key)
}

/// Anchor reference position for Lattice / FFD deformations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LatticeAnchor {
    #[default]
    Center,
    MinLeft,
    MaxRight,
}

/// Free-Form Deformation (FFD) 2D Lattice Cage parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatticeCage {
    /// Multiplier on duration / time spacing (1.0 = 100% no change, 1.5 = stretch 50%).
    pub time_scale: f64,
    /// Multiplier on value amplitude (1.0 = 100%, 2.0 = amplify, -1.0 = invert).
    pub value_scale: f64,
    /// Absolute time shift in seconds.
    pub time_offset: f64,
    /// Absolute value shift.
    pub value_offset: f64,
    /// Time skew factor proportional to vertical position (creates progressive acceleration/retiming).
    pub time_skew: f64,
    /// Anchor point for scaling.
    pub anchor: LatticeAnchor,
}

impl Default for LatticeCage {
    fn default() -> Self {
        Self {
            time_scale: 1.0,
            value_scale: 1.0,
            time_offset: 0.0,
            value_offset: 0.0,
            time_skew: 0.0,
            anchor: LatticeAnchor::Center,
        }
    }
}

impl LatticeCage {
    /// Applies FFD lattice transformation to selected keyframes across channels.
    pub fn transform_keys(&self, keys: &mut [KeyPoint]) {
        if keys.is_empty() {
            return;
        }

        // Determine bounding box of selected keys
        let mut t_min = f64::MAX;
        let mut t_max = f64::MIN;
        let mut v_min = f64::MAX;
        let mut v_max = f64::MIN;

        for k in keys.iter() {
            t_min = t_min.min(k.time);
            t_max = t_max.max(k.time);
            v_min = v_min.min(k.value);
            v_max = v_max.max(k.value);
        }

        let dt = (t_max - t_min).max(EPSILON);
        let dv = (v_max - v_min).max(EPSILON);

        let (anchor_t, anchor_v) = match self.anchor {
            LatticeAnchor::MinLeft => (t_min, v_min),
            LatticeAnchor::MaxRight => (t_max, v_max),
            LatticeAnchor::Center => ((t_min + t_max) * 0.5, (v_min + v_max) * 0.5),
        };

        for k in keys.iter_mut() {
            let norm_v = (k.value - v_min) / dv;

            // Retime and transform time
            let delta_t = k.time - anchor_t;
            let skewed_t = delta_t * self.time_scale + (norm_v - 0.5) * self.time_skew * dt;
            k.time = anchor_t + skewed_t + self.time_offset;

            // Scale and shift value
            let delta_v = k.value - anchor_v;
            k.value = anchor_v + delta_v * self.value_scale + self.value_offset;

            // Adjust tangent speeds inversely to time scale and proportional to value scale
            if self.time_scale.abs() > EPSILON {
                let speed_ratio = self.value_scale / self.time_scale;
                k.in_tangent.speed *= speed_ratio;
                k.out_tangent.speed *= speed_ratio;
            }
        }
    }
}

/// Multi-channel graph editor display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewMode {
    /// Normalized mode: all channels mapped to 0.0 .. 1.0 vertical scale.
    #[default]
    Normalized,
    /// Absolute mode: raw physical values.
    Absolute,
    /// Stacked mode: channels partitioned into vertical lanes.
    Stacked,
    /// Speed graph mode: first derivative (units/sec).
    SpeedGraph,
}

/// Buffer snapshot for comparing before/after curve states.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferSnapshot {
    pub channels: Vec<ChannelCurve>,
}

impl BufferSnapshot {
    pub fn capture(channels: &[ChannelCurve]) -> Self {
        Self {
            channels: channels.to_vec(),
        }
    }

    /// Computes Root-Mean-Square Error (RMSE) between snapshot and current curves.
    pub fn compute_delta_rmse(&self, current: &[ChannelCurve], samples_per_channel: usize) -> f64 {
        if self.channels.is_empty() || current.is_empty() {
            return 0.0;
        }

        let mut total_sq_diff = 0.0;
        let mut total_samples = 0;

        for old_ch in &self.channels {
            if let Some(new_ch) = current.iter().find(|c| c.id == old_ch.id) {
                let (t0, t1, _, _) = old_ch.bounds();
                let steps = samples_per_channel.max(2);
                let step_dt = (t1 - t0) / (steps as f64 - 1.0);

                for s in 0..steps {
                    let t = t0 + s as f64 * step_dt;
                    let v_old = old_ch.eval(t);
                    let v_new = new_ch.eval(t);
                    let diff = v_new - v_old;
                    total_sq_diff += diff * diff;
                    total_samples += 1;
                }
            }
        }

        if total_samples > 0 {
            (total_sq_diff / total_samples as f64).sqrt()
        } else {
            0.0
        }
    }
}

/// Preset easing curve configurations.
pub struct PresetTangents;

impl PresetTangents {
    /// Easy Ease: 33.33% influence, 0.0 speed.
    pub fn easy_ease() -> (TangentHandle, TangentHandle) {
        (
            TangentHandle::new(33.333, 0.0),
            TangentHandle::new(33.333, 0.0),
        )
    }

    /// Strong Punch: 75% influence, 0.0 speed.
    pub fn strong_punch() -> (TangentHandle, TangentHandle) {
        (
            TangentHandle::new(75.0, 0.0),
            TangentHandle::new(75.0, 0.0),
        )
    }

    /// Extreme Snap: 88% influence, 0.0 speed.
    pub fn extreme_snap() -> (TangentHandle, TangentHandle) {
        (
            TangentHandle::new(88.0, 0.0),
            TangentHandle::new(88.0, 0.0),
        )
    }

    /// Anticipation & Overshoot: back-curving tangents.
    pub fn overshoot() -> (TangentHandle, TangentHandle) {
        (
            TangentHandle::new(45.0, -0.85),
            TangentHandle::new(33.333, 1.2),
        )
    }
}
