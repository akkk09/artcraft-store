//! VectorCraft VectorFirstAid document and vector cleanup tool (VectorCraft ABI v1).
//! Inspired by Astute Graphics VectorFirstAid.

use serde_json::{Value, json};

pub const MANIFEST: &str = r#"{
  "id": "org.vectorcraft.community.vectorfirstaid",
  "name": "Vector FirstAid",
  "version": "1.0.0",
  "kind": "filter",
  "author": "ArtCraft Store community",
  "description": "Document and vector cleanup tool for VectorCraft: removes unpainted paths, redundant and stray anchor points, empty clipping masks, empty groups, and cleans up text artifacts.",
  "params": {
    "remove_unpainted": {"type": "bool", "default": true},
    "remove_stray_points": {"type": "bool", "default": true},
    "reduce_redundant_points": {"type": "bool", "default": true},
    "collinear_threshold": {"type": "number", "min": 0.01, "max": 10.0, "default": 0.5},
    "remove_unnecessary_clips": {"type": "bool", "default": true},
    "remove_empty_groups": {"type": "bool", "default": true},
    "clean_text": {"type": "bool", "default": true},
    "join_endpoints": {"type": "bool", "default": false},
    "join_distance": {"type": "number", "min": 0.0, "max": 50.0, "default": 1.0}
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

#[derive(Debug, Clone)]
pub struct VectorFirstAidParams {
    pub remove_unpainted: bool,
    pub remove_stray_points: bool,
    pub reduce_redundant_points: bool,
    pub collinear_threshold: f64, // degrees
    pub remove_unnecessary_clips: bool,
    pub remove_empty_groups: bool,
    pub clean_text: bool,
    pub join_endpoints: bool,
    pub join_distance: f64,
}

impl Default for VectorFirstAidParams {
    fn default() -> Self {
        Self {
            remove_unpainted: true,
            remove_stray_points: true,
            reduce_redundant_points: true,
            collinear_threshold: 0.5,
            remove_unnecessary_clips: true,
            remove_empty_groups: true,
            clean_text: true,
            join_endpoints: false,
            join_distance: 1.0,
        }
    }
}

impl VectorFirstAidParams {
    pub fn from_json(val: &Value) -> Self {
        let mut p = Self::default();
        if let Some(v) = val.get("remove_unpainted").and_then(|v| v.as_bool()) {
            p.remove_unpainted = v;
        }
        if let Some(v) = val.get("remove_stray_points").and_then(|v| v.as_bool()) {
            p.remove_stray_points = v;
        }
        if let Some(v) = val.get("reduce_redundant_points").and_then(|v| v.as_bool()) {
            p.reduce_redundant_points = v;
        }
        if let Some(v) = val.get("collinear_threshold").and_then(|v| v.as_f64()) {
            p.collinear_threshold = v.clamp(0.01, 10.0);
        }
        if let Some(v) = val.get("remove_unnecessary_clips").and_then(|v| v.as_bool()) {
            p.remove_unnecessary_clips = v;
        }
        if let Some(v) = val.get("remove_empty_groups").and_then(|v| v.as_bool()) {
            p.remove_empty_groups = v;
        }
        if let Some(v) = val.get("clean_text").and_then(|v| v.as_bool()) {
            p.clean_text = v;
        }
        if let Some(v) = val.get("join_endpoints").and_then(|v| v.as_bool()) {
            p.join_endpoints = v;
        }
        if let Some(v) = val.get("join_distance").and_then(|v| v.as_f64()) {
            p.join_distance = v.max(0.0);
        }
        p
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn dist(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Checks if paint definition has visible color/opacity.
fn is_paint_visible(paint: &Value) -> bool {
    if let Value::Object(map) = paint {
        if let Some(t) = map.get("type").and_then(|s| s.as_str()) {
            if t == "none" || t == "null" {
                return false;
            }
        }
        if let Some(op) = map.get("opacity").and_then(|v| v.as_f64()) {
            if op <= 1e-4 {
                return false;
            }
        }
        true
    } else {
        false
    }
}

/// Returns whether an object has any visible paint (fill, stroke, raster image, or text).
pub fn is_object_painted(map: &serde_json::Map<String, Value>) -> bool {
    // 1. Check direct raster image
    if map.contains_key("image") || map.contains_key("raster") {
        return true;
    }

    // 2. Check text
    if map.contains_key("text") || map.contains_key("content") {
        return true;
    }

    // 3. Check kind type
    if let Some(kind) = map.get("kind").and_then(|k| k.as_object()) {
        if let Some(t) = kind.get("type").and_then(|s| s.as_str()) {
            if t == "image" || t == "text" {
                return true;
            }
        }
    }

    // 4. Check appearance.items
    if let Some(app) = map.get("appearance").and_then(|a| a.as_object()) {
        if let Some(items) = app.get("items").and_then(|i| i.as_array()) {
            for item in items {
                if let Some(kind) = item.get("kind").and_then(|k| k.as_str()) {
                    if kind == "fill" {
                        if let Some(paint) = item.get("paint") {
                            if is_paint_visible(paint) {
                                return true;
                            }
                        }
                    } else if kind == "stroke" {
                        let width = item.get("width").and_then(|w| w.as_f64()).unwrap_or(1.0);
                        if width > 1e-4 {
                            if let Some(paint) = item.get("paint") {
                                if is_paint_visible(paint) {
                                    return true;
                                }
                            } else {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    // 5. Check fills array
    if let Some(fills) = map.get("fills").and_then(|f| f.as_array()) {
        for fill in fills {
            if is_paint_visible(fill) {
                return true;
            }
        }
    }

    // 6. Check single fill
    if let Some(fill) = map.get("fill") {
        if is_paint_visible(fill) {
            return true;
        }
    }

    // 7. Check strokes array
    if let Some(strokes) = map.get("strokes").and_then(|s| s.as_array()) {
        for stroke in strokes {
            let width = stroke.get("width").and_then(|w| w.as_f64()).unwrap_or(1.0);
            if width > 1e-4 {
                if let Some(paint) = stroke.get("paint") {
                    if is_paint_visible(paint) {
                        return true;
                    }
                } else if is_paint_visible(stroke) {
                    return true;
                }
            }
        }
    }

    // 8. Check single stroke
    if let Some(stroke) = map.get("stroke") {
        let width = stroke.get("width").and_then(|w| w.as_f64()).unwrap_or(1.0);
        if width > 1e-4 {
            if let Some(paint) = stroke.get("paint") {
                if is_paint_visible(paint) {
                    return true;
                }
            } else if is_paint_visible(stroke) {
                return true;
            }
        }
    }

    // 9. If object is a group with children, considered painted if any child is painted
    if let Some(children) = map.get("children").and_then(|c| c.as_array()) {
        if children.iter().any(|c| if let Value::Object(m) = c { is_object_painted(m) } else { false }) {
            return true;
        }
    }

    false
}

/// Checks if an anchor has curved handles (deflection from point position).
fn anchor_has_curves(anchor: &Value) -> bool {
    if let Value::Object(anc) = anchor {
        let p = match anc.get("p").and_then(|v| v.as_array()) {
            Some(arr) if arr.len() >= 2 => Point2D::new(arr[0].as_f64().unwrap_or(0.0), arr[1].as_f64().unwrap_or(0.0)),
            _ => return false,
        };
        if let Some(in_h) = anc.get("in").and_then(|v| v.as_array()) {
            if in_h.len() >= 2 {
                let h = Point2D::new(in_h[0].as_f64().unwrap_or(0.0), in_h[1].as_f64().unwrap_or(0.0));
                if h.dist(p) > 1e-3 {
                    return true;
                }
            }
        }
        if let Some(out_h) = anc.get("out").and_then(|v| v.as_array()) {
            if out_h.len() >= 2 {
                let h = Point2D::new(out_h[0].as_f64().unwrap_or(0.0), out_h[1].as_f64().unwrap_or(0.0));
                if h.dist(p) > 1e-3 {
                    return true;
                }
            }
        }
    }
    false
}

fn parse_anchor_pt(anchor: &Value) -> Option<Point2D> {
    if let Value::Object(anc) = anchor {
        let arr = anc.get("p")?.as_array()?;
        if arr.len() >= 2 {
            return Some(Point2D::new(arr[0].as_f64()?, arr[1].as_f64()?));
        }
    } else if let Value::Array(arr) = anchor {
        if arr.len() >= 2 {
            return Some(Point2D::new(arr[0].as_f64()?, arr[1].as_f64()?));
        }
    }
    None
}

/// Reduces duplicate adjacent nodes and collinear nodes along an anchor list.
pub fn simplify_anchors(anchors: &[Value], closed: bool, collinear_threshold_deg: f64) -> Vec<Value> {
    if anchors.len() <= 2 {
        return anchors.to_vec();
    }

    // Step 1: Remove adjacent duplicate points
    let mut step1: Vec<Value> = Vec::with_capacity(anchors.len());
    for anc in anchors {
        if let Some(curr_pt) = parse_anchor_pt(anc) {
            if let Some(last) = step1.last() {
                if let Some(last_pt) = parse_anchor_pt(last) {
                    if last_pt.dist(curr_pt) < 1e-4 && !anchor_has_curves(anc) {
                        continue; // duplicate adjacent anchor
                    }
                }
            }
        }
        step1.push(anc.clone());
    }

    let min_needed = if closed { 3 } else { 2 };
    if step1.len() <= min_needed {
        return step1;
    }

    // Step 2: Remove redundant collinear nodes
    let threshold_rad = collinear_threshold_deg.to_radians();
    let mut result: Vec<Value> = Vec::with_capacity(step1.len());
    let n = step1.len();

    for i in 0..n {
        let is_endpoint = !closed && (i == 0 || i == n - 1);
        if is_endpoint {
            result.push(step1[i].clone());
            continue;
        }

        let curr = &step1[i];
        if anchor_has_curves(curr) {
            result.push(curr.clone());
            continue;
        }

        let prev_idx = if i == 0 { n - 1 } else { i - 1 };
        let next_idx = (i + 1) % n;

        let prev_pt = match parse_anchor_pt(&step1[prev_idx]) {
            Some(p) => p,
            None => { result.push(curr.clone()); continue; }
        };
        let curr_pt = match parse_anchor_pt(curr) {
            Some(p) => p,
            None => { result.push(curr.clone()); continue; }
        };
        let next_pt = match parse_anchor_pt(&step1[next_idx]) {
            Some(p) => p,
            None => { result.push(curr.clone()); continue; }
        };

        let u_x = curr_pt.x - prev_pt.x;
        let u_y = curr_pt.y - prev_pt.y;
        let v_x = next_pt.x - curr_pt.x;
        let v_y = next_pt.y - curr_pt.y;

        let u_len = (u_x * u_x + u_y * u_y).sqrt();
        let v_len = (v_x * v_x + v_y * v_y).sqrt();

        if u_len < 1e-6 || v_len < 1e-6 {
            // Degenerate segment, can drop
            continue;
        }

        let dot = (u_x * v_x + u_y * v_y) / (u_len * v_len);
        let dot_clamped = dot.clamp(-1.0, 1.0);
        let angle = dot_clamped.acos();

        // If angle deviation is within threshold, the point is collinear
        if angle <= threshold_rad {
            // collinear redundant node, skip adding
            continue;
        }

        result.push(curr.clone());
    }

    if result.len() < min_needed {
        step1
    } else {
        result
    }
}

/// Simplifies a simple points array `[[x,y], ...]`
pub fn simplify_points_array(pts: &[Value], closed: bool, collinear_threshold_deg: f64) -> Vec<Value> {
    if pts.len() <= 2 {
        return pts.to_vec();
    }
    let threshold_rad = collinear_threshold_deg.to_radians();
    let mut step1: Vec<Value> = Vec::with_capacity(pts.len());

    for item in pts {
        if let Some(curr_pt) = parse_anchor_pt(item) {
            if let Some(last) = step1.last() {
                if let Some(last_pt) = parse_anchor_pt(last) {
                    if last_pt.dist(curr_pt) < 1e-4 {
                        continue;
                    }
                }
            }
        }
        step1.push(item.clone());
    }

    let min_needed = if closed { 3 } else { 2 };
    if step1.len() <= min_needed {
        return step1;
    }

    let mut result: Vec<Value> = Vec::with_capacity(step1.len());
    let n = step1.len();

    for i in 0..n {
        let is_endpoint = !closed && (i == 0 || i == n - 1);
        if is_endpoint {
            result.push(step1[i].clone());
            continue;
        }

        let prev_pt = match parse_anchor_pt(&step1[if i == 0 { n - 1 } else { i - 1 }]) {
            Some(p) => p,
            None => { result.push(step1[i].clone()); continue; }
        };
        let curr_pt = match parse_anchor_pt(&step1[i]) {
            Some(p) => p,
            None => { result.push(step1[i].clone()); continue; }
        };
        let next_pt = match parse_anchor_pt(&step1[(i + 1) % n]) {
            Some(p) => p,
            None => { result.push(step1[i].clone()); continue; }
        };

        let u_x = curr_pt.x - prev_pt.x;
        let u_y = curr_pt.y - prev_pt.y;
        let v_x = next_pt.x - curr_pt.x;
        let v_y = next_pt.y - curr_pt.y;

        let u_len = (u_x * u_x + u_y * u_y).sqrt();
        let v_len = (v_x * v_x + v_y * v_y).sqrt();

        if u_len < 1e-6 || v_len < 1e-6 {
            continue;
        }

        let dot = (u_x * v_x + u_y * v_y) / (u_len * v_len);
        let angle = dot.clamp(-1.0, 1.0).acos();

        if angle <= threshold_rad {
            continue;
        }

        result.push(step1[i].clone());
    }

    if result.len() < min_needed {
        step1
    } else {
        result
    }
}

/// Total count of anchors across subpaths or points.
pub fn count_object_anchors_in_map(map: &serde_json::Map<String, Value>) -> usize {
    if let Some(pts) = map.get("points").and_then(|p| p.as_array()) {
        return pts.len();
    }

    let subpaths_opt = map.get("path")
        .and_then(|p| p.get("subpaths"))
        .or_else(|| map.get("kind").and_then(|k| k.get("path")).and_then(|p| p.get("subpaths")))
        .and_then(|s| s.as_array());

    if let Some(subpaths) = subpaths_opt {
        let mut total = 0;
        for sp in subpaths {
            if let Some(anchors) = sp.get("anchors").and_then(|a| a.as_array()) {
                total += anchors.len();
            }
        }
        return total;
    }
    0
}

/// Applies VectorFirstAid cleanup to a single object or returns None if object should be removed.
pub fn clean_single_object(mut obj: Value, params: &VectorFirstAidParams) -> Option<Value> {
    let Value::Object(ref mut map) = obj else {
        return Some(obj);
    };

    // 1. Text object cleanup
    let is_text = map.contains_key("text")
        || map.contains_key("content")
        || map.get("kind").and_then(|k| k.get("type")).and_then(|t| t.as_str()) == Some("text");

    if is_text && params.clean_text {
        let text_key = if map.contains_key("text") {
            Some("text")
        } else if map.contains_key("content") {
            Some("content")
        } else {
            None
        };
        if let Some(key) = text_key {
            if let Some(text_val) = map.get_mut(key) {
                if let Some(s) = text_val.as_str() {
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        return None; // empty stray text
                    }
                    *text_val = json!(trimmed);
                }
            }
        }
        return Some(obj);
    }

    // 2. Unnecessary / empty clipping mask cleanup
    let is_clip = map.get("clip").and_then(|c| c.as_bool()).unwrap_or(false)
        || map.get("clipping_path").and_then(|c| c.as_bool()).unwrap_or(false)
        || map.get("kind").and_then(|k| k.get("type")).and_then(|t| t.as_str()) == Some("clip");

    if is_clip && params.remove_unnecessary_clips {
        let children = map.get("children").and_then(|c| c.as_array());
        match children {
            Some(arr) if arr.is_empty() => return None, // clips nothing
            None => {
                // Empty clipping path with no content
                if !map.contains_key("children") && !map.contains_key("objects") {
                    return None;
                }
            }
            _ => {}
        }
    }

    // 3. Group cleanup
    if let Some(children_val) = map.get_mut("children").and_then(|c| c.as_array_mut()) {
        let mut cleaned_children = Vec::new();
        for child in children_val.drain(..) {
            if let Some(cleaned) = clean_single_object(child, params) {
                cleaned_children.push(cleaned);
            }
        }
        if params.remove_empty_groups && cleaned_children.is_empty() {
            return None; // Prune empty group
        }
        *children_val = cleaned_children;
        return Some(obj);
    }

    // 4. Stray Points removal (<= 1 anchor)
    let anchor_count = count_object_anchors_in_map(map);
    let is_path = map.contains_key("points")
        || map.contains_key("path")
        || map.get("kind").and_then(|k| k.get("type")).and_then(|t| t.as_str()) == Some("path");

    if is_path && params.remove_stray_points && anchor_count <= 1 {
        return None; // stray point
    }

    // 5. Unpainted path removal
    if is_path && params.remove_unpainted && !is_object_painted(map) {
        return None; // unpainted ghost path
    }

    // 6. Redundant points reduction & endpoint joining on path anchors
    if params.reduce_redundant_points || params.join_endpoints {
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
                let mut closed = sp.get("closed").and_then(|c| c.as_bool()).unwrap_or(false);

                // Join endpoints if within threshold
                if params.join_endpoints && !closed {
                    if let Some(anchors) = sp.get("anchors").and_then(|a| a.as_array()) {
                        if anchors.len() >= 2 {
                            if let (Some(first), Some(last)) = (anchors.first(), anchors.last()) {
                                if let (Some(p1), Some(p2)) = (parse_anchor_pt(first), parse_anchor_pt(last)) {
                                    if p1.dist(p2) <= params.join_distance {
                                        closed = true;
                                        if let Value::Object(sp_map) = sp {
                                            sp_map.insert("closed".to_string(), json!(true));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if params.reduce_redundant_points {
                    if let Some(anchors) = sp.get_mut("anchors").and_then(|a| a.as_array_mut()) {
                        *anchors = simplify_anchors(anchors, closed, params.collinear_threshold);
                    }
                }
            }
        }

        // Mode B: points array
        let mut closed = map.get("closed").and_then(|c| c.as_bool()).unwrap_or(false);
        if let Some(pts) = map.get("points").and_then(|p| p.as_array()) {
            if params.join_endpoints && !closed && pts.len() >= 2 {
                if let (Some(first), Some(last)) = (pts.first(), pts.last()) {
                    if let (Some(p1), Some(p2)) = (parse_anchor_pt(first), parse_anchor_pt(last)) {
                        if p1.dist(p2) <= params.join_distance {
                            closed = true;
                        }
                    }
                }
            }
        }
        if closed {
            map.insert("closed".to_string(), json!(true));
        }

        if params.reduce_redundant_points {
            if let Some(pts) = map.get_mut("points").and_then(|p| p.as_array_mut()) {
                *pts = simplify_points_array(pts, closed, params.collinear_threshold);
            }
        }
    }

    Some(obj)
}

/// Applies VectorFirstAid cleanup to a full VectorCraft document.
pub fn apply_vectorfirstaid(doc: &mut Value, params: &VectorFirstAidParams) {
    if let Some(objects) = doc.get_mut("objects").and_then(|o| o.as_array_mut()) {
        let mut final_objects = Vec::new();
        for obj in objects.drain(..) {
            if let Some(cleaned) = clean_single_object(obj, params) {
                final_objects.push(cleaned);
            }
        }
        *objects = final_objects;
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
    let p = VectorFirstAidParams::from_json(&params_val);

    apply_vectorfirstaid(&mut doc, &p);

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
    fn test_remove_unpainted_path() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "points": [[0.0, 0.0], [10.0, 10.0], [20.0, 0.0]],
                    // No fills, no strokes
                },
                {
                    "id": 2,
                    "points": [[0.0, 0.0], [50.0, 50.0]],
                    "fills": [{"type": "solid", "color": {"model": "rgb", "r": 1.0, "g": 0.0, "b": 0.0}}]
                }
            ]
        });

        let params = VectorFirstAidParams {
            remove_unpainted: true,
            ..Default::default()
        };

        apply_vectorfirstaid(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0]["id"], 2);
    }

    #[test]
    fn test_remove_stray_point() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "points": [[15.0, 25.0]], // Single point stray
                    "fills": [{"type": "solid", "color": {"model": "rgb", "r": 0.0, "g": 1.0, "b": 0.0}}]
                },
                {
                    "id": 2,
                    "points": [[0.0, 0.0], [100.0, 100.0]],
                    "fills": [{"type": "solid", "color": {"model": "rgb", "r": 0.0, "g": 1.0, "b": 0.0}}]
                }
            ]
        });

        let params = VectorFirstAidParams {
            remove_stray_points: true,
            ..Default::default()
        };

        apply_vectorfirstaid(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0]["id"], 2);
    }

    #[test]
    fn test_reduce_collinear_nodes() {
        // Point (50, 0) is perfectly collinear between (0, 0) and (100, 0)
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "points": [[0.0, 0.0], [50.0, 0.0], [100.0, 0.0]],
                    "closed": false,
                    "strokes": [{"width": 2.0, "paint": {"type": "solid"}}]
                }
            ]
        });

        let params = VectorFirstAidParams {
            reduce_redundant_points: true,
            collinear_threshold: 0.5,
            ..Default::default()
        };

        apply_vectorfirstaid(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        assert_eq!(objs.len(), 1);
        let pts = objs[0]["points"].as_array().unwrap();
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0], json!([0.0, 0.0]));
        assert_eq!(pts[1], json!([100.0, 0.0]));
    }

    #[test]
    fn test_join_endpoints() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "points": [[0.0, 0.0], [10.0, 50.0], [0.5, 0.5]], // End point is within 1.0 of (0,0)
                    "closed": false,
                    "strokes": [{"width": 1.0, "paint": {"type": "solid"}}]
                }
            ]
        });

        let params = VectorFirstAidParams {
            join_endpoints: true,
            join_distance: 1.0,
            reduce_redundant_points: false,
            ..Default::default()
        };

        apply_vectorfirstaid(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        assert_eq!(objs[0]["closed"], json!(true));
    }

    #[test]
    fn test_clean_empty_text_and_empty_groups() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "kind": {"type": "text"},
                    "text": "    " // Whitespace only
                },
                {
                    "id": 2,
                    "type": "group",
                    "children": [] // Empty group
                },
                {
                    "id": 3,
                    "kind": {"type": "text"},
                    "text": "  Hello VectorCraft  "
                }
            ]
        });

        let params = VectorFirstAidParams {
            clean_text: true,
            remove_empty_groups: true,
            ..Default::default()
        };

        apply_vectorfirstaid(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0]["id"], 3);
        assert_eq!(objs[0]["text"], json!("Hello VectorCraft"));
    }
}
