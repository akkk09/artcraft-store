//! VectorCraft Reform path sculpting and reshaping plugin (VectorCraft ABI v1).
//! Inspired by Astute Graphics Reform.

use std::f64::consts::PI;
use serde_json::{Value, json};

pub const MANIFEST: &str = r#"{
  "id": "org.vectorcraft.community.reform",
  "name": "Vector Reform",
  "version": "1.0.0",
  "kind": "filter",
  "author": "ArtCraft Store community",
  "description": "Organic path reshaping and sculpting tool for VectorCraft: re-form, bulge, pinch, bend, and smooth paths and path segments without managing individual Bézier handles.",
  "params": {
    "mode": {"type": "choice", "options": ["bulge", "pinch", "bend", "push", "taper", "smooth"], "default": "bulge"},
    "amount": {"type": "number", "min": -500.0, "max": 500.0, "default": 25.0},
    "range_start": {"type": "number", "min": 0.0, "max": 1.0, "default": 0.0},
    "range_end": {"type": "number", "min": 0.0, "max": 1.0, "default": 1.0},
    "falloff": {"type": "choice", "options": ["smooth", "linear", "sharp"], "default": "smooth"},
    "tension": {"type": "number", "min": 0.0, "max": 2.0, "default": 1.0},
    "preserve_endpoints": {"type": "bool", "default": true}
  }
}"#;

#[no_mangle]
pub extern "C" fn vc_abi_version() -> u32 {
    1
}

fn pack(slice: &[u8]) -> u64 {
    let ptr = slice.as_ptr() as usize as u64;
    let len = slice.len() as u64;
    (len << 32) | (ptr & 0xFFFF_FFFF)
}

#[no_mangle]
pub extern "C" fn vc_manifest() -> u64 {
    pack(MANIFEST.as_bytes())
}

#[no_mangle]
pub extern "C" fn vc_alloc(size: u32) -> u32 {
    let mut block = Vec::<u8>::with_capacity(size as usize);
    let ptr = block.as_mut_ptr();
    std::mem::forget(block);
    ptr as usize as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReformMode {
    Bulge,
    Pinch,
    Bend,
    Push,
    Taper,
    Smooth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalloffType {
    Smooth,
    Linear,
    Sharp,
}

#[derive(Debug, Clone)]
pub struct ReformParams {
    pub mode: ReformMode,
    pub amount: f64,
    pub range_start: f64,
    pub range_end: f64,
    pub falloff: FalloffType,
    pub tension: f64,
    pub preserve_endpoints: bool,
}

impl Default for ReformParams {
    fn default() -> Self {
        Self {
            mode: ReformMode::Bulge,
            amount: 25.0,
            range_start: 0.0,
            range_end: 1.0,
            falloff: FalloffType::Smooth,
            tension: 1.0,
            preserve_endpoints: true,
        }
    }
}

impl ReformParams {
    pub fn from_json(val: &Value) -> Self {
        let mut p = Self::default();
        if let Some(s) = val.get("mode").and_then(|v| v.as_str()) {
            p.mode = match s {
                "pinch" => ReformMode::Pinch,
                "bend" => ReformMode::Bend,
                "push" => ReformMode::Push,
                "taper" => ReformMode::Taper,
                "smooth" => ReformMode::Smooth,
                _ => ReformMode::Bulge,
            };
        }
        if let Some(v) = val.get("amount").and_then(|v| v.as_f64()) {
            p.amount = v;
        }
        if let Some(v) = val.get("range_start").and_then(|v| v.as_f64()) {
            p.range_start = v.clamp(0.0, 1.0);
        }
        if let Some(v) = val.get("range_end").and_then(|v| v.as_f64()) {
            p.range_end = v.clamp(0.0, 1.0);
        }
        if p.range_start > p.range_end {
            std::mem::swap(&mut p.range_start, &mut p.range_end);
        }
        if let Some(s) = val.get("falloff").and_then(|v| v.as_str()) {
            p.falloff = match s {
                "linear" => FalloffType::Linear,
                "sharp" => FalloffType::Sharp,
                _ => FalloffType::Smooth,
            };
        }
        if let Some(v) = val.get("tension").and_then(|v| v.as_f64()) {
            p.tension = v.clamp(0.0, 2.0);
        }
        if let Some(v) = val.get("preserve_endpoints").and_then(|v| v.as_bool()) {
            p.preserve_endpoints = v;
        }
        p
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }

    pub fn sub(self, other: Self) -> Self {
        Self { x: self.x - other.x, y: self.y - other.y }
    }

    pub fn scale(self, s: f64) -> Self {
        Self { x: self.x * s, y: self.y * s }
    }

    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 1e-9 {
            Self { x: self.x / len, y: self.y / len }
        } else {
            Self { x: 0.0, y: 0.0 }
        }
    }

    pub fn normal(self) -> Self {
        // Left perpendicular normal
        Self { x: -self.y, y: self.x }
    }
}

#[derive(Debug, Clone)]
pub struct AnchorNode {
    pub p: Vec2,
    pub in_handle: Vec2,
    pub out_handle: Vec2,
}

impl AnchorNode {
    pub fn from_value(val: &Value) -> Option<Self> {
        if let Value::Object(map) = val {
            let p_arr = map.get("p")?.as_array()?;
            if p_arr.len() < 2 { return None; }
            let p = Vec2::new(p_arr[0].as_f64()?, p_arr[1].as_f64()?);

            let in_h = if let Some(arr) = map.get("in").and_then(|v| v.as_array()) {
                if arr.len() >= 2 {
                    Vec2::new(arr[0].as_f64().unwrap_or(p.x), arr[1].as_f64().unwrap_or(p.y))
                } else { p }
            } else { p };

            let out_h = if let Some(arr) = map.get("out").and_then(|v| v.as_array()) {
                if arr.len() >= 2 {
                    Vec2::new(arr[0].as_f64().unwrap_or(p.x), arr[1].as_f64().unwrap_or(p.y))
                } else { p }
            } else { p };

            Some(Self { p, in_handle: in_h, out_handle: out_h })
        } else if let Value::Array(arr) = val {
            if arr.len() >= 2 {
                let p = Vec2::new(arr[0].as_f64()?, arr[1].as_f64()?);
                Some(Self { p, in_handle: p, out_handle: p })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn to_value(&self) -> Value {
        json!({
            "p": [self.p.x, self.p.y],
            "in": [self.in_handle.x, self.in_handle.y],
            "out": [self.out_handle.x, self.out_handle.y]
        })
    }
}

fn compute_falloff_weight(u: f64, falloff: FalloffType) -> f64 {
    let u_clamped = u.clamp(0.0, 1.0);
    match falloff {
        FalloffType::Smooth => {
            // Half-sine bell curve from 0 to 1 back to 0
            (PI * u_clamped).sin()
        }
        FalloffType::Linear => {
            1.0 - 2.0 * (u_clamped - 0.5).abs()
        }
        FalloffType::Sharp => {
            let s = (PI * u_clamped).sin();
            s * s
        }
    }
}

/// Reform polyline / anchor sequence with sculpted deformation.
pub fn reform_anchors(nodes: &[AnchorNode], closed: bool, params: &ReformParams) -> Vec<AnchorNode> {
    let n = nodes.len();
    if n < 2 {
        return nodes.to_vec();
    }

    // 1. Compute cumulative chord lengths
    let mut cum_len = Vec::with_capacity(n);
    cum_len.push(0.0);
    let mut total_len = 0.0;
    for i in 1..n {
        let seg = nodes[i].p.sub(nodes[i - 1].p).length();
        total_len += seg;
        cum_len.push(total_len);
    }
    if closed {
        total_len += nodes[0].p.sub(nodes[n - 1].p).length();
    }
    if total_len < 1e-6 {
        return nodes.to_vec();
    }

    let range_span = (params.range_end - params.range_start).max(1e-4);
    let mut deformed: Vec<AnchorNode> = nodes.to_vec();

    // 2. Compute deformation for each anchor
    for i in 0..n {
        let is_endpoint = !closed && (i == 0 || i == n - 1);
        if is_endpoint && params.preserve_endpoints {
            continue;
        }

        let t = cum_len[i] / total_len;
        if t < params.range_start || t > params.range_end {
            continue;
        }

        let local_u = (t - params.range_start) / range_span;
        let weight = compute_falloff_weight(local_u, params.falloff);
        if weight <= 1e-6 {
            continue;
        }

        // Compute local tangent & normal
        let prev_idx = if i == 0 { if closed { n - 1 } else { 0 } } else { i - 1 };
        let next_idx = if i == n - 1 { if closed { 0 } else { n - 1 } } else { i + 1 };

        let tangent = nodes[next_idx].p.sub(nodes[prev_idx].p).normalize();
        let normal = tangent.normal();

        let displacement = match params.mode {
            ReformMode::Bulge => normal.scale(params.amount * weight),
            ReformMode::Pinch => normal.scale(-params.amount * weight),
            ReformMode::Bend => {
                let chord = nodes[n - 1].p.sub(nodes[0].p).normalize();
                chord.normal().scale(params.amount * weight)
            }
            ReformMode::Push => {
                // Directional vertical push
                Vec2::new(0.0, params.amount * weight)
            }
            ReformMode::Taper => {
                let center_x = (nodes[0].p.x + nodes[n - 1].p.x) * 0.5;
                let dx = nodes[i].p.x - center_x;
                let scale_factor = (params.amount * 0.02 * local_u).clamp(-0.9, 5.0);
                Vec2::new(dx * scale_factor, 0.0)
            }
            ReformMode::Smooth => {
                // Laplacian smoothing
                let avg = nodes[prev_idx].p.add(nodes[next_idx].p).scale(0.5);
                avg.sub(nodes[i].p).scale(params.tension * weight * 0.5)
            }
        };

        deformed[i].p = deformed[i].p.add(displacement);
        // Also offset handles by same translation to preserve local curvature
        deformed[i].in_handle = deformed[i].in_handle.add(displacement);
        deformed[i].out_handle = deformed[i].out_handle.add(displacement);
    }

    // 3. Smooth tangent handles for reshaped nodes using tension parameter
    for i in 0..n {
        let is_endpoint = !closed && (i == 0 || i == n - 1);
        if is_endpoint && params.preserve_endpoints {
            continue;
        }

        let prev_idx = if i == 0 { if closed { n - 1 } else { 0 } } else { i - 1 };
        let next_idx = if i == n - 1 { if closed { 0 } else { n - 1 } } else { i + 1 };

        let v_prev = deformed[i].p.sub(deformed[prev_idx].p);
        let v_next = deformed[next_idx].p.sub(deformed[i].p);
        let d_prev = v_prev.length();
        let d_next = v_next.length();

        let dir = deformed[next_idx].p.sub(deformed[prev_idx].p).normalize();
        let handle_scale = params.tension * 0.33;

        deformed[i].in_handle = deformed[i].p.sub(dir.scale(d_prev * handle_scale));
        deformed[i].out_handle = deformed[i].p.add(dir.scale(d_next * handle_scale));
    }

    deformed
}

/// Applies Reform to a single object map.
pub fn apply_reform_to_object(map: &mut serde_json::Map<String, Value>, params: &ReformParams) {
    // Mode A: path.subpaths or kind.path.subpaths
    let mut subpaths_target = if let Some(path_obj) = map.get_mut("path").and_then(|p| p.as_object_mut()) {
        path_obj.get_mut("subpaths").and_then(|s| s.as_array_mut())
    } else if let Some(kind_obj) = map.get_mut("kind").and_then(|k| k.as_object_mut()) {
        kind_obj.get_mut("path").and_then(|p| p.get_mut("subpaths")).and_then(|s| s.as_array_mut())
    } else {
        None
    };

    if let Some(subpaths) = subpaths_target.as_deref_mut() {
        for sp in subpaths.iter_mut() {
            let closed = sp.get("closed").and_then(|c| c.as_bool()).unwrap_or(false);
            if let Some(anchors_val) = sp.get_mut("anchors").and_then(|a| a.as_array_mut()) {
                let nodes: Vec<AnchorNode> = anchors_val.iter().filter_map(AnchorNode::from_value).collect();
                if nodes.len() >= 2 {
                    let reformed = reform_anchors(&nodes, closed, params);
                    *anchors_val = reformed.into_iter().map(|n| n.to_value()).collect();
                }
            }
        }
    }

    // Mode B: points array `[[x, y], ...]`
    let closed_pt = map.get("closed").and_then(|c| c.as_bool()).unwrap_or(false);
    if let Some(pts_val) = map.get_mut("points").and_then(|p| p.as_array_mut()) {
        let nodes: Vec<AnchorNode> = pts_val.iter().filter_map(AnchorNode::from_value).collect();
        if nodes.len() >= 2 {
            let reformed = reform_anchors(&nodes, closed_pt, params);
            *pts_val = reformed.into_iter().map(|n| json!([n.p.x, n.p.y])).collect();
        }
    }

    // Mode C: recursive groups
    if let Some(children) = map.get_mut("children").and_then(|c| c.as_array_mut()) {
        for child in children {
            if let Value::Object(child_map) = child {
                apply_reform_to_object(child_map, params);
            }
        }
    }
}

pub fn apply_reform(doc: &mut Value, params: &ReformParams) {
    if let Some(objects) = doc.get_mut("objects").and_then(|o| o.as_array_mut()) {
        for obj in objects {
            if let Value::Object(map) = obj {
                apply_reform_to_object(map, params);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn vc_run(input: *const u8, input_len: u32, params: *const u8, params_len: u32) -> i64 {
    let in_slice = unsafe { std::slice::from_raw_parts(input, input_len as usize) };
    let param_slice = unsafe { std::slice::from_raw_parts(params, params_len as usize) };

    let mut doc: Value = match serde_json::from_slice(in_slice) {
        Ok(v) => v,
        Err(_) => return -1,
    };
    let params_val: Value = serde_json::from_slice(param_slice).unwrap_or(Value::Null);
    let p = ReformParams::from_json(&params_val);

    apply_reform(&mut doc, &p);

    let out_bytes = match serde_json::to_vec(&doc) {
        Ok(b) => b,
        Err(_) => return -2,
    };
    let out_slice = out_bytes.leak();
    pack(out_slice) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulge_deformation() {
        // Horizontal straight line from (0, 0) to (100, 0) with a midpoint at (50, 0)
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "points": [[0.0, 0.0], [50.0, 0.0], [100.0, 0.0]],
                    "closed": false
                }
            ]
        });

        let params = ReformParams {
            mode: ReformMode::Bulge,
            amount: 30.0,
            range_start: 0.0,
            range_end: 1.0,
            preserve_endpoints: true,
            ..Default::default()
        };

        apply_reform(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        let pts = objs[0]["points"].as_array().unwrap();

        // Endpoints (0, 0) and (100, 0) must be preserved
        assert_eq!(pts[0], json!([0.0, 0.0]));
        assert_eq!(pts[2], json!([100.0, 0.0]));

        // Midpoint at u=0.5 must be displaced along normal by amount=30.0
        // Tangent for (0,0)->(100,0) is (1, 0), normal is (0, 1) or (-0, 1)
        let mid_y = pts[1][1].as_f64().unwrap();
        assert!((mid_y.abs() - 30.0).abs() < 1e-3);
    }

    #[test]
    fn test_pinch_deformation() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "points": [[0.0, 0.0], [50.0, 0.0], [100.0, 0.0]],
                    "closed": false
                }
            ]
        });

        let params = ReformParams {
            mode: ReformMode::Pinch,
            amount: 20.0,
            range_start: 0.0,
            range_end: 1.0,
            preserve_endpoints: true,
            ..Default::default()
        };

        apply_reform(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        let pts = objs[0]["points"].as_array().unwrap();

        let mid_y = pts[1][1].as_f64().unwrap();
        assert!((mid_y - (-20.0)).abs() < 1e-3);
    }

    #[test]
    fn test_subpath_anchors_and_handles() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "path": {
                        "subpaths": [
                            {
                                "closed": false,
                                "anchors": [
                                    {"p": [0.0, 0.0], "in": [0.0, 0.0], "out": [0.0, 0.0]},
                                    {"p": [50.0, 0.0], "in": [40.0, 0.0], "out": [60.0, 0.0]},
                                    {"p": [100.0, 0.0], "in": [100.0, 0.0], "out": [100.0, 0.0]}
                                ]
                            }
                        ]
                    }
                }
            ]
        });

        let params = ReformParams {
            mode: ReformMode::Bulge,
            amount: 25.0,
            tension: 1.0,
            preserve_endpoints: true,
            ..Default::default()
        };

        apply_reform(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        let anchors = objs[0]["path"]["subpaths"][0]["anchors"].as_array().unwrap();

        // Mid anchor position must have moved
        let mid_p = anchors[1]["p"].as_array().unwrap();
        assert!((mid_p[1].as_f64().unwrap().abs() - 25.0).abs() < 1e-3);

        // Handles must be populated
        assert!(anchors[1].get("in").is_some());
        assert!(anchors[1].get("out").is_some());
    }

    #[test]
    fn test_laplacian_smooth_mode() {
        // Jagged path with zig-zag center: (0, 0), (50, 40), (100, 0)
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "points": [[0.0, 0.0], [50.0, 40.0], [100.0, 0.0]],
                    "closed": false
                }
            ]
        });

        let params = ReformParams {
            mode: ReformMode::Smooth,
            tension: 1.0,
            preserve_endpoints: true,
            ..Default::default()
        };

        apply_reform(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        let pts = objs[0]["points"].as_array().unwrap();

        // Midpoint Y should have smoothed down from 40 towards 0 (the chord average)
        let smoothed_y = pts[1][1].as_f64().unwrap();
        assert!(smoothed_y < 40.0 && smoothed_y >= 0.0);
    }
}
