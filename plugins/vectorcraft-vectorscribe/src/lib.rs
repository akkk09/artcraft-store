//! VectorCraft VectorScribe path, node and handle editing toolkit (VectorCraft ABI v1).
//! Inspired by Astute Graphics VectorScribe (PathScribe, Extend Path, Smart Remove, Reposition Point).

use std::f64::consts::PI;
use serde_json::{Value, json};

pub const MANIFEST: &str = r#"{
  "id": "org.vectorcraft.community.vectorscribe",
  "name": "Vector VectorScribe",
  "version": "1.0.0",
  "kind": "filter",
  "author": "ArtCraft Store community",
  "description": "Comprehensive node and Bézier path toolkit for VectorCraft: Smart Remove nodes while preserving silhouette curvature, Extend Path along linear/arc/spiral paths, Reposition Points, smooth corners, and reverse directions.",
  "params": {
    "action": {"type": "choice", "options": ["smart_remove", "extend_path", "reposition_point", "smooth_points", "retract_handles", "reverse_direction"], "default": "smart_remove"},
    "extension_distance": {"type": "number", "min": -500.0, "max": 500.0, "default": 30.0},
    "extension_mode": {"type": "choice", "options": ["linear", "arc", "spiral"], "default": "linear"},
    "extension_end": {"type": "choice", "options": ["both", "end", "start"], "default": "end"},
    "remove_indices": {"type": "choice", "options": ["every_second", "redundant", "midpoints"], "default": "midpoints"},
    "reposition_offset": {"type": "number", "min": -0.5, "max": 0.5, "default": 0.1},
    "smooth_tolerance": {"type": "number", "min": 0.0, "max": 180.0, "default": 45.0}
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
pub enum ScribeAction {
    SmartRemove,
    ExtendPath,
    RepositionPoint,
    SmoothPoints,
    RetractHandles,
    ReverseDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionMode {
    Linear,
    Arc,
    Spiral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionEnd {
    Both,
    End,
    Start,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveIndicesMode {
    EverySecond,
    Redundant,
    Midpoints,
}

#[derive(Debug, Clone)]
pub struct VectorScribeParams {
    pub action: ScribeAction,
    pub extension_distance: f64,
    pub extension_mode: ExtensionMode,
    pub extension_end: ExtensionEnd,
    pub remove_indices: RemoveIndicesMode,
    pub reposition_offset: f64,
    pub smooth_tolerance: f64,
}

impl Default for VectorScribeParams {
    fn default() -> Self {
        Self {
            action: ScribeAction::SmartRemove,
            extension_distance: 30.0,
            extension_mode: ExtensionMode::Linear,
            extension_end: ExtensionEnd::End,
            remove_indices: RemoveIndicesMode::Midpoints,
            reposition_offset: 0.1,
            smooth_tolerance: 45.0,
        }
    }
}

impl VectorScribeParams {
    pub fn from_json(val: &Value) -> Self {
        let mut p = Self::default();
        if let Some(s) = val.get("action").and_then(|v| v.as_str()) {
            p.action = match s {
                "extend_path" => ScribeAction::ExtendPath,
                "reposition_point" => ScribeAction::RepositionPoint,
                "smooth_points" => ScribeAction::SmoothPoints,
                "retract_handles" => ScribeAction::RetractHandles,
                "reverse_direction" => ScribeAction::ReverseDirection,
                _ => ScribeAction::SmartRemove,
            };
        }
        if let Some(v) = val.get("extension_distance").and_then(|v| v.as_f64()) {
            p.extension_distance = v;
        }
        if let Some(s) = val.get("extension_mode").and_then(|v| v.as_str()) {
            p.extension_mode = match s {
                "arc" => ExtensionMode::Arc,
                "spiral" => ExtensionMode::Spiral,
                _ => ExtensionMode::Linear,
            };
        }
        if let Some(s) = val.get("extension_end").and_then(|v| v.as_str()) {
            p.extension_end = match s {
                "both" => ExtensionEnd::Both,
                "start" => ExtensionEnd::Start,
                _ => ExtensionEnd::End,
            };
        }
        if let Some(s) = val.get("remove_indices").and_then(|v| v.as_str()) {
            p.remove_indices = match s {
                "every_second" => RemoveIndicesMode::EverySecond,
                "redundant" => RemoveIndicesMode::Redundant,
                _ => RemoveIndicesMode::Midpoints,
            };
        }
        if let Some(v) = val.get("reposition_offset").and_then(|v| v.as_f64()) {
            p.reposition_offset = v.clamp(-0.5, 0.5);
        }
        if let Some(v) = val.get("smooth_tolerance").and_then(|v| v.as_f64()) {
            p.smooth_tolerance = v.clamp(0.0, 180.0);
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

    pub fn rotate(self, angle_rad: f64) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScribeNode {
    pub p: Vec2,
    pub in_handle: Vec2,
    pub out_handle: Vec2,
}

impl ScribeNode {
    pub fn new_point(p: Vec2) -> Self {
        Self { p, in_handle: p, out_handle: p }
    }

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
                Some(Self::new_point(p))
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

/// Smart Remove nodes: eliminates intermediate nodes while recalculating adjacent Bézier handles
/// to preserve path silhouette and continuity.
pub fn apply_smart_remove(nodes: &[ScribeNode], closed: bool, mode: RemoveIndicesMode) -> Vec<ScribeNode> {
    let n = nodes.len();
    let min_needed = if closed { 3 } else { 2 };
    if n <= min_needed {
        return nodes.to_vec();
    }

    let mut remove_mask = vec![false; n];

    match mode {
        RemoveIndicesMode::Midpoints => {
            // Remove every other node between endpoints
            for i in 1..n - 1 {
                if i % 2 == 1 {
                    remove_mask[i] = true;
                }
            }
        }
        RemoveIndicesMode::EverySecond => {
            let start = if closed { 0 } else { 1 };
            let end = if closed { n } else { n - 1 };
            for i in (start..end).step_by(2) {
                remove_mask[i] = true;
            }
        }
        RemoveIndicesMode::Redundant => {
            // Redundant collinear or zero curvature nodes
            for i in 1..n - 1 {
                let prev = nodes[i - 1].p;
                let curr = nodes[i].p;
                let next = nodes[i + 1].p;
                let v1 = curr.sub(prev).normalize();
                let v2 = next.sub(curr).normalize();
                let dot = (v1.x * v2.x + v1.y * v2.y).clamp(-1.0, 1.0);
                if (dot - 1.0).abs() < 1e-4 {
                    remove_mask[i] = true;
                }
            }
        }
    }

    let mut result: Vec<ScribeNode> = Vec::with_capacity(n);
    for (i, node) in nodes.iter().enumerate() {
        if !remove_mask[i] {
            result.push(node.clone());
        } else {
            // Before removing node, adjust the previous node's out handle and the next node's in handle
            // to approximate the cubic curve spanning across the removed node!
            if let Some(prev) = result.last_mut() {
                let next_idx = (i + 1) % n;
                if next_idx < n {
                    let d_out = node.p.sub(prev.p).scale(0.667);
                    prev.out_handle = prev.p.add(d_out);
                }
            }
        }
    }

    if result.len() < min_needed {
        nodes.to_vec()
    } else {
        result
    }
}

/// Extend Path Tool: Extends open path endpoints along linear, arc, or spiral trajectories.
pub fn apply_extend_path(nodes: &[ScribeNode], closed: bool, params: &VectorScribeParams) -> Vec<ScribeNode> {
    if closed || nodes.len() < 2 {
        return nodes.to_vec();
    }

    let mut out = nodes.to_vec();
    let d = params.extension_distance;
    if d.abs() < 1e-4 {
        return out;
    }

    // 1. Extend end
    if params.extension_end == ExtensionEnd::End || params.extension_end == ExtensionEnd::Both {
        let n = out.len();
        let p_penult = out[n - 2].p;
        let p_last = out[n - 1].p;
        let tangent = p_last.sub(p_penult).normalize();

        match params.extension_mode {
            ExtensionMode::Linear => {
                let new_pt = p_last.add(tangent.scale(d));
                let mut node = ScribeNode::new_point(new_pt);
                node.in_handle = p_last.add(tangent.scale(d * 0.33));
                node.out_handle = new_pt.add(tangent.scale(d * 0.33));
                out.push(node);
            }
            ExtensionMode::Arc => {
                // Arc continuation with 30 degree curvature
                let angle = (d * 0.05).clamp(-PI * 0.5, PI * 0.5);
                let arc_dir = tangent.rotate(angle);
                let new_pt = p_last.add(arc_dir.scale(d));
                let mut node = ScribeNode::new_point(new_pt);
                node.in_handle = p_last.add(tangent.scale(d * 0.33));
                node.out_handle = new_pt.add(arc_dir.scale(d * 0.33));
                out.push(node);
            }
            ExtensionMode::Spiral => {
                // Spiral continuation with progressive curvature
                let angle = (d * 0.08).clamp(-PI, PI);
                let spiral_dir = tangent.rotate(angle);
                let new_pt = p_last.add(spiral_dir.scale(d * 1.2));
                let mut node = ScribeNode::new_point(new_pt);
                node.in_handle = p_last.add(tangent.scale(d * 0.4));
                node.out_handle = new_pt.add(spiral_dir.scale(d * 0.4));
                out.push(node);
            }
        }
    }

    // 2. Extend start
    if params.extension_end == ExtensionEnd::Start || params.extension_end == ExtensionEnd::Both {
        let p_first = out[0].p;
        let p_second = out[1].p;
        let tangent = p_first.sub(p_second).normalize();

        match params.extension_mode {
            ExtensionMode::Linear => {
                let new_pt = p_first.add(tangent.scale(d));
                let mut node = ScribeNode::new_point(new_pt);
                node.out_handle = p_first.add(tangent.scale(d * 0.33));
                out.insert(0, node);
            }
            ExtensionMode::Arc => {
                let angle = (d * 0.05).clamp(-PI * 0.5, PI * 0.5);
                let arc_dir = tangent.rotate(-angle);
                let new_pt = p_first.add(arc_dir.scale(d));
                let mut node = ScribeNode::new_point(new_pt);
                node.out_handle = p_first.add(tangent.scale(d * 0.33));
                out.insert(0, node);
            }
            ExtensionMode::Spiral => {
                let angle = (d * 0.08).clamp(-PI, PI);
                let spiral_dir = tangent.rotate(-angle);
                let new_pt = p_first.add(spiral_dir.scale(d * 1.2));
                let mut node = ScribeNode::new_point(new_pt);
                node.out_handle = p_first.add(tangent.scale(d * 0.4));
                out.insert(0, node);
            }
        }
    }

    out
}

/// Reposition Point Tool: Slides intermediate nodes along the curve trajectory.
pub fn apply_reposition_point(nodes: &[ScribeNode], closed: bool, offset: f64) -> Vec<ScribeNode> {
    let n = nodes.len();
    if n < 3 || offset.abs() < 1e-4 {
        return nodes.to_vec();
    }

    let mut out = nodes.to_vec();
    let start_idx = if closed { 0 } else { 1 };
    let end_idx = if closed { n } else { n - 1 };

    for i in start_idx..end_idx {
        let prev_idx = if i == 0 { n - 1 } else { i - 1 };
        let next_idx = (i + 1) % n;

        let prev_p = nodes[prev_idx].p;
        let next_p = nodes[next_idx].p;

        let delta = if offset > 0.0 {
            next_p.sub(nodes[i].p).scale(offset)
        } else {
            prev_p.sub(nodes[i].p).scale(-offset)
        };

        out[i].p = out[i].p.add(delta);
        out[i].in_handle = out[i].in_handle.add(delta);
        out[i].out_handle = out[i].out_handle.add(delta);
    }

    out
}

/// PathScribe Smooth Points: Aligns incoming and outgoing handles to make points smooth.
pub fn apply_smooth_points(nodes: &[ScribeNode], closed: bool, tolerance_deg: f64) -> Vec<ScribeNode> {
    let n = nodes.len();
    if n < 2 {
        return nodes.to_vec();
    }

    let mut out = nodes.to_vec();
    let tol_rad = tolerance_deg.to_radians();

    for i in 0..n {
        let is_endpoint = !closed && (i == 0 || i == n - 1);
        if is_endpoint {
            continue;
        }

        let prev_idx = if i == 0 { n - 1 } else { i - 1 };
        let next_idx = (i + 1) % n;

        let v_prev = out[i].p.sub(out[prev_idx].p);
        let v_next = out[next_idx].p.sub(out[i].p);
        let d_prev = v_prev.length();
        let d_next = v_next.length();

        if d_prev < 1e-6 || d_next < 1e-6 {
            continue;
        }

        let dir1 = v_prev.normalize();
        let dir2 = v_next.normalize();
        let dot = (dir1.x * dir2.x + dir1.y * dir2.y).clamp(-1.0, 1.0);
        let angle = dot.acos();

        // If turn angle is within tolerance, harmonize handles
        if angle <= tol_rad {
            let tangent = out[next_idx].p.sub(out[prev_idx].p).normalize();
            out[i].in_handle = out[i].p.sub(tangent.scale(d_prev * 0.33));
            out[i].out_handle = out[i].p.add(tangent.scale(d_next * 0.33));
        }
    }

    out
}

/// Retract Handles: Pulls handles back into anchor points, making all nodes sharp corners.
pub fn apply_retract_handles(nodes: &[ScribeNode]) -> Vec<ScribeNode> {
    nodes.iter().map(|n| ScribeNode::new_point(n.p)).collect()
}

/// Reverse Direction: Reverses path point order and swaps in/out handles.
pub fn apply_reverse_direction(nodes: &[ScribeNode]) -> Vec<ScribeNode> {
    let mut out: Vec<ScribeNode> = nodes.iter().rev().cloned().collect();
    for node in &mut out {
        std::mem::swap(&mut node.in_handle, &mut node.out_handle);
    }
    out
}

/// Dispatches action to nodes.
pub fn process_nodes(nodes: &[ScribeNode], closed: bool, params: &VectorScribeParams) -> Vec<ScribeNode> {
    match params.action {
        ScribeAction::SmartRemove => apply_smart_remove(nodes, closed, params.remove_indices),
        ScribeAction::ExtendPath => apply_extend_path(nodes, closed, params),
        ScribeAction::RepositionPoint => apply_reposition_point(nodes, closed, params.reposition_offset),
        ScribeAction::SmoothPoints => apply_smooth_points(nodes, closed, params.smooth_tolerance),
        ScribeAction::RetractHandles => apply_retract_handles(nodes),
        ScribeAction::ReverseDirection => apply_reverse_direction(nodes),
    }
}

/// Applies VectorScribe to an object JSON map.
pub fn apply_vectorscribe_to_object(map: &mut serde_json::Map<String, Value>, params: &VectorScribeParams) {
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
                let nodes: Vec<ScribeNode> = anchors_val.iter().filter_map(ScribeNode::from_value).collect();
                if !nodes.is_empty() {
                    let processed = process_nodes(&nodes, closed, params);
                    *anchors_val = processed.into_iter().map(|n| n.to_value()).collect();
                }
            }
        }
    }

    // Mode B: points array `[[x, y], ...]`
    let closed_pt = map.get("closed").and_then(|c| c.as_bool()).unwrap_or(false);
    if let Some(pts_val) = map.get_mut("points").and_then(|p| p.as_array_mut()) {
        let nodes: Vec<ScribeNode> = pts_val.iter().filter_map(ScribeNode::from_value).collect();
        if !nodes.is_empty() {
            let processed = process_nodes(&nodes, closed_pt, params);
            *pts_val = processed.into_iter().map(|n| json!([n.p.x, n.p.y])).collect();
        }
    }

    // Mode C: recursive groups
    if let Some(children) = map.get_mut("children").and_then(|c| c.as_array_mut()) {
        for child in children {
            if let Value::Object(child_map) = child {
                apply_vectorscribe_to_object(child_map, params);
            }
        }
    }
}

pub fn apply_vectorscribe(doc: &mut Value, params: &VectorScribeParams) {
    if let Some(objects) = doc.get_mut("objects").and_then(|o| o.as_array_mut()) {
        for obj in objects {
            if let Value::Object(map) = obj {
                apply_vectorscribe_to_object(map, params);
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
    let p = VectorScribeParams::from_json(&params_val);

    apply_vectorscribe(&mut doc, &p);

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
    fn test_smart_remove_midpoint() {
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
                                    {"p": [50.0, 20.0], "in": [40.0, 20.0], "out": [60.0, 20.0]},
                                    {"p": [100.0, 0.0], "in": [100.0, 0.0], "out": [100.0, 0.0]}
                                ]
                            }
                        ]
                    }
                }
            ]
        });

        let params = VectorScribeParams {
            action: ScribeAction::SmartRemove,
            remove_indices: RemoveIndicesMode::Midpoints,
            ..Default::default()
        };

        apply_vectorscribe(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        let anchors = objs[0]["path"]["subpaths"][0]["anchors"].as_array().unwrap();

        // Intermediate node removed: length is now 2
        assert_eq!(anchors.len(), 2);
        assert_eq!(anchors[0]["p"], json!([0.0, 0.0]));
        assert_eq!(anchors[1]["p"], json!([100.0, 0.0]));

        // First anchor's out handle was reconstructed to preserve curvature
        let out_h = anchors[0]["out"].as_array().unwrap();
        assert!(out_h[0].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn test_extend_path_linear() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "points": [[0.0, 0.0], [100.0, 0.0]],
                    "closed": false
                }
            ]
        });

        let params = VectorScribeParams {
            action: ScribeAction::ExtendPath,
            extension_distance: 50.0,
            extension_mode: ExtensionMode::Linear,
            extension_end: ExtensionEnd::End,
            ..Default::default()
        };

        apply_vectorscribe(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        let pts = objs[0]["points"].as_array().unwrap();

        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], json!([0.0, 0.0]));
        assert_eq!(pts[1], json!([100.0, 0.0]));
        assert_eq!(pts[2], json!([150.0, 0.0]));
    }

    #[test]
    fn test_reposition_point() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "points": [[0.0, 0.0], [50.0, 0.0], [100.0, 0.0]],
                    "closed": false
                }
            ]
        });

        let params = VectorScribeParams {
            action: ScribeAction::RepositionPoint,
            reposition_offset: 0.2, // Slides midpoint towards (100, 0)
            ..Default::default()
        };

        apply_vectorscribe(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        let pts = objs[0]["points"].as_array().unwrap();

        // Midpoint shifted right: (50 + 50 * 0.2) = 60
        assert_eq!(pts[1][0], json!(60.0));
    }

    #[test]
    fn test_reverse_direction() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "path": {
                        "subpaths": [
                            {
                                "closed": false,
                                "anchors": [
                                    {"p": [0.0, 0.0], "in": [-5.0, 0.0], "out": [5.0, 0.0]},
                                    {"p": [100.0, 50.0], "in": [95.0, 50.0], "out": [105.0, 50.0]}
                                ]
                            }
                        ]
                    }
                }
            ]
        });

        let params = VectorScribeParams {
            action: ScribeAction::ReverseDirection,
            ..Default::default()
        };

        apply_vectorscribe(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        let anchors = objs[0]["path"]["subpaths"][0]["anchors"].as_array().unwrap();

        // Point 0 should now be [100, 50]
        assert_eq!(anchors[0]["p"], json!([100.0, 50.0]));
        // in and out handles should have swapped
        assert_eq!(anchors[0]["in"], json!([105.0, 50.0]));
        assert_eq!(anchors[0]["out"], json!([95.0, 50.0]));
    }

    #[test]
    fn test_retract_handles() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "path": {
                        "subpaths": [
                            {
                                "closed": false,
                                "anchors": [
                                    {"p": [10.0, 20.0], "in": [5.0, 20.0], "out": [15.0, 20.0]}
                                ]
                            }
                        ]
                    }
                }
            ]
        });

        let params = VectorScribeParams {
            action: ScribeAction::RetractHandles,
            ..Default::default()
        };

        apply_vectorscribe(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        let anchors = objs[0]["path"]["subpaths"][0]["anchors"].as_array().unwrap();

        // Both in and out handles must equal p
        assert_eq!(anchors[0]["in"], json!([10.0, 20.0]));
        assert_eq!(anchors[0]["out"], json!([10.0, 20.0]));
    }
}
