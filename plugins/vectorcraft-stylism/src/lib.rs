//! VectorCraft Stylism vector styling plugin (VectorCraft ABI v1).
//! Inspired by Astute Graphics Stylism.

use serde_json::{Value, json};
use std::f64::consts::PI;

pub const MANIFEST: &str = r#"{
  "id": "org.vectorcraft.community.stylism",
  "name": "Vector Stylism",
  "version": "1.0.0",
  "kind": "filter",
  "author": "ArtCraft Store community",
  "description": "Live vector styling, path offset, drop shadows, and multi-contour glow effects for VectorCraft.",
  "params": {
    "offset_distance": {"type": "number", "min": -200.0, "max": 200.0, "default": 0.0},
    "join": {"type": "choice", "options": ["round", "miter", "bevel"], "default": "round"},
    "miter_limit": {"type": "number", "min": 1.0, "max": 10.0, "default": 4.0},
    "shadow_dx": {"type": "number", "min": -1000.0, "max": 1000.0, "default": 5.0},
    "shadow_dy": {"type": "number", "min": -1000.0, "max": 1000.0, "default": 5.0},
    "shadow_blur": {"type": "number", "min": 0.0, "max": 50.0, "default": 8.0},
    "shadow_opacity": {"type": "number", "min": 0.0, "max": 1.0, "default": 0.4},
    "shadow_enabled": {"type": "bool", "default": false},
    "glow_radius": {"type": "number", "min": 0.0, "max": 50.0, "default": 0.0},
    "glow_opacity": {"type": "number", "min": 0.0, "max": 1.0, "default": 0.6},
    "glow_enabled": {"type": "bool", "default": false}
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
pub enum JoinType {
    Round,
    Miter,
    Bevel,
}

#[derive(Debug, Clone)]
pub struct StylismParams {
    pub offset_distance: f64,
    pub join: JoinType,
    pub miter_limit: f64,
    pub shadow_dx: f64,
    pub shadow_dy: f64,
    pub shadow_blur: f64,
    pub shadow_opacity: f64,
    pub shadow_enabled: bool,
    pub glow_radius: f64,
    pub glow_opacity: f64,
    pub glow_enabled: bool,
}

impl StylismParams {
    pub fn from_json(val: &Value) -> Self {
        let join_str = val.get("join").and_then(|v| v.as_str()).unwrap_or("round");
        let join = match join_str {
            "miter" => JoinType::Miter,
            "bevel" => JoinType::Bevel,
            _ => JoinType::Round,
        };

        Self {
            offset_distance: val.get("offset_distance").and_then(|v| v.as_f64()).unwrap_or(0.0),
            join,
            miter_limit: val.get("miter_limit").and_then(|v| v.as_f64()).unwrap_or(4.0),
            shadow_dx: val.get("shadow_dx").and_then(|v| v.as_f64()).unwrap_or(5.0),
            shadow_dy: val.get("shadow_dy").and_then(|v| v.as_f64()).unwrap_or(5.0),
            shadow_blur: val.get("shadow_blur").and_then(|v| v.as_f64()).unwrap_or(8.0),
            shadow_opacity: val.get("shadow_opacity").and_then(|v| v.as_f64()).unwrap_or(0.4),
            shadow_enabled: val.get("shadow_enabled").and_then(|v| v.as_bool()).unwrap_or(false),
            glow_radius: val.get("glow_radius").and_then(|v| v.as_f64()).unwrap_or(0.0),
            glow_opacity: val.get("glow_opacity").and_then(|v| v.as_f64()).unwrap_or(0.6),
            glow_enabled: val.get("glow_enabled").and_then(|v| v.as_bool()).unwrap_or(false),
        }
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

    pub fn sub(self, other: Self) -> Self {
        Self { x: self.x - other.x, y: self.y - other.y }
    }

    pub fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
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
        // Left normal: (-y, x)
        Self { x: -self.y, y: self.x }
    }
}

pub fn signed_area(pts: &[Vec2]) -> f64 {
    let mut area = 0.0;
    let n = pts.len();
    for i in 0..n {
        let j = (i + 1) % n;
        area += pts[i].x * pts[j].y - pts[j].x * pts[i].y;
    }
    area * 0.5
}

/// Offsets a polygon/path point sequence by distance `d`.
pub fn offset_polyline(points: &[Vec2], d: f64, join: JoinType, miter_limit: f64, is_closed: bool) -> Vec<Vec2> {
    let n = points.len();
    if n < 2 || d.abs() < 1e-6 {
        return points.to_vec();
    }

    let effective_d = if is_closed && n >= 3 {
        let area = signed_area(points);
        if area > 0.0 { -d } else { d }
    } else {
        d
    };

    let mut normals = Vec::with_capacity(n);
    let count = if is_closed { n } else { n - 1 };

    for i in 0..count {
        let next_idx = (i + 1) % n;
        let seg = points[next_idx].sub(points[i]);
        normals.push(seg.normalize().normal());
    }

    let mut result = Vec::new();

    for i in 0..n {
        let (n_prev, n_next) = if is_closed {
            let prev_idx = if i == 0 { n - 1 } else { i - 1 };
            (normals[prev_idx], normals[i])
        } else {
            if i == 0 {
                let p = points[0].add(normals[0].scale(effective_d));
                result.push(p);
                continue;
            } else if i == n - 1 {
                let p = points[n - 1].add(normals[n - 2].scale(effective_d));
                result.push(p);
                continue;
            } else {
                (normals[i - 1], normals[i])
            }
        };

        let pt = points[i];
        let p_prev = pt.add(n_prev.scale(effective_d));
        let p_next = pt.add(n_next.scale(effective_d));

        let dot = n_prev.x * n_next.x + n_prev.y * n_next.y;
        let is_collinear = (dot - 1.0).abs() < 1e-4;

        if is_collinear {
            result.push(p_prev);
            continue;
        }

        match join {
            JoinType::Bevel => {
                result.push(p_prev);
                result.push(p_next);
            }
            JoinType::Miter => {
                let miter_vec = n_prev.add(n_next).normalize();
                let sin_half = (1.0 + dot) / 2.0;
                let miter_len = if sin_half > 1e-4 {
                    effective_d / sin_half.sqrt()
                } else {
                    effective_d * miter_limit
                };

                if (miter_len / effective_d.abs()).abs() <= miter_limit {
                    result.push(pt.add(miter_vec.scale(miter_len)));
                } else {
                    result.push(p_prev);
                    result.push(p_next);
                }
            }
            JoinType::Round => {
                result.push(p_prev);
                // Insert 2 intermediate arc vertices for smooth corner
                let angle_prev = n_prev.y.atan2(n_prev.x);
                let mut angle_next = n_next.y.atan2(n_next.x);
                if (angle_next - angle_prev).abs() > PI {
                    if angle_next > angle_prev { angle_next -= 2.0 * PI; }
                    else { angle_next += 2.0 * PI; }
                }
                for step in 1..=2 {
                    let t = step as f64 / 3.0;
                    let angle = angle_prev + t * (angle_next - angle_prev);
                    let arc_pt = pt.add(Vec2::new(angle.cos() * effective_d, angle.sin() * effective_d));
                    result.push(arc_pt);
                }
                result.push(p_next);
            }
        }
    }

    result
}

fn points_from_val(val: &Value) -> Option<Vec<Vec2>> {
    let arr = val.as_array()?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        if let Value::Array(xy) = item {
            if xy.len() >= 2 {
                let x = xy[0].as_f64()?;
                let y = xy[1].as_f64()?;
                out.push(Vec2::new(x, y));
            }
        } else if let Value::Object(coord) = item {
            let x = coord.get("x")?.as_f64()?;
            let y = coord.get("y")?.as_f64()?;
            out.push(Vec2::new(x, y));
        }
    }
    Some(out)
}

fn val_from_points(points: &[Vec2]) -> Value {
    let list: Vec<Value> = points.iter().map(|p| json!([p.x, p.y])).collect();
    Value::Array(list)
}

pub fn apply_offset_to_object(obj: &mut Value, d: f64, join: JoinType, miter_limit: f64) {
    if let Value::Object(map) = obj {
        if let Some(path_val) = map.get_mut("path") {
            if let Some(subpaths) = path_val.get_mut("subpaths").and_then(|s| s.as_array_mut()) {
                for subpath in subpaths {
                    let closed = subpath.get("closed").and_then(|v| v.as_bool()).unwrap_or(true);
                    if let Some(anchors) = subpath.get_mut("anchors").and_then(|a| a.as_array_mut()) {
                        let pts: Vec<Vec2> = anchors.iter().filter_map(|anc| {
                            let p = anc.get("p")?.as_array()?;
                            Some(Vec2::new(p.get(0)?.as_f64()?, p.get(1)?.as_f64()?))
                        }).collect();
                        if pts.len() >= 2 {
                            let offset_pts = offset_polyline(&pts, d, join, miter_limit, closed);
                            if offset_pts.len() == pts.len() {
                                for (anc, new_p) in anchors.iter_mut().zip(offset_pts.iter()) {
                                    if let Some(Value::Array(p_arr)) = anc.get_mut("p") {
                                        p_arr[0] = json!(new_p.x);
                                        p_arr[1] = json!(new_p.y);
                                    }
                                }
                            } else {
                                *anchors = offset_pts.into_iter().map(|pt| {
                                    json!({
                                        "p": [pt.x, pt.y],
                                        "in": [pt.x, pt.y],
                                        "out": [pt.x, pt.y]
                                    })
                                }).collect();
                            }
                        }
                    }
                }
            }
        }
        if let Some(pts_val) = map.get("points") {
            if let Some(pts) = points_from_val(pts_val) {
                let closed = map.get("closed").and_then(|v| v.as_bool()).unwrap_or(true);
                let offset_pts = offset_polyline(&pts, d, join, miter_limit, closed);
                map.insert("points".to_string(), val_from_points(&offset_pts));
            }
        }
    }
}

pub fn create_drop_shadow_object(obj: &Value, dx: f64, dy: f64, opacity: f64) -> Value {
    let mut shadow = obj.clone();
    if let Value::Object(map) = &mut shadow {
        map.remove("id");

        if let Some(path_val) = map.get_mut("path") {
            if let Some(subpaths) = path_val.get_mut("subpaths").and_then(|s| s.as_array_mut()) {
                for subpath in subpaths {
                    if let Some(anchors) = subpath.get_mut("anchors").and_then(|a| a.as_array_mut()) {
                        for anchor in anchors {
                            for key in &["p", "in", "out"] {
                                if let Some(Value::Array(arr)) = anchor.get_mut(*key) {
                                    if arr.len() >= 2 {
                                        if let (Some(x), Some(y)) = (arr[0].as_f64(), arr[1].as_f64()) {
                                            arr[0] = json!(x + dx);
                                            arr[1] = json!(y + dy);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Shift points by (dx, dy)
        if let Some(pts_val) = map.get_mut("points") {
            if let Some(mut pts) = points_from_val(pts_val) {
                for p in &mut pts {
                    p.x += dx;
                    p.y += dy;
                }
                *pts_val = val_from_points(&pts);
            }
        }
        let new_x = map.get("x").and_then(|v| v.as_f64()).map(|x| x + dx);
        let new_y = map.get("y").and_then(|v| v.as_f64()).map(|y| y + dy);
        if let (Some(x), Some(y)) = (new_x, new_y) {
            map.insert("x".to_string(), json!(x));
            map.insert("y".to_string(), json!(y));
        }

        let shadow_paint = json!({
            "type": "solid",
            "color": {
                "model": "rgb",
                "r": 0.0,
                "g": 0.0,
                "b": 0.0
            },
            "opacity": opacity
        });
        map.insert("fills".to_string(), json!([shadow_paint.clone()]));
        map.insert("fill".to_string(), shadow_paint);
        map.remove("strokes");
        map.remove("stroke");
    }
    shadow
}

pub fn create_glow_object(obj: &Value, radius: f64, opacity: f64) -> Value {
    let mut glow = obj.clone();
    apply_offset_to_object(&mut glow, radius, JoinType::Round, 4.0);

    if let Value::Object(map) = &mut glow {
        map.remove("id");

        let glow_paint = json!({
            "type": "solid",
            "color": {
                "model": "rgb",
                "r": 1.0,
                "g": 0.88,
                "b": 0.2
            },
            "opacity": opacity
        });
        map.insert("fills".to_string(), json!([glow_paint.clone()]));
        map.insert("fill".to_string(), glow_paint);
        map.remove("strokes");
        map.remove("stroke");
    }
    glow
}

pub fn apply_stylism(doc: &mut Value, params: &StylismParams) {
    if let Some(objects) = doc.get_mut("objects").and_then(|o| o.as_array_mut()) {
        let mut final_objects = Vec::new();

        for mut obj in objects.drain(..) {
            // 1. Add Drop Shadow underneath if enabled
            if params.shadow_enabled {
                let shadow = create_drop_shadow_object(&obj, params.shadow_dx, params.shadow_dy, params.shadow_opacity);
                final_objects.push(shadow);
            }

            // 2. Add Outer Glow underneath if enabled
            if params.glow_enabled && params.glow_radius > 0.0 {
                let glow = create_glow_object(&obj, params.glow_radius, params.glow_opacity);
                final_objects.push(glow);
            }

            // 3. Apply Offset to primary artwork if distance is set
            if params.offset_distance.abs() > 1e-4 {
                apply_offset_to_object(&mut obj, params.offset_distance, params.join, params.miter_limit);
            }

            final_objects.push(obj);
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
    let p = StylismParams::from_json(&params_val);

    apply_stylism(&mut doc, &p);

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
    fn test_offset_polyline_expansion() {
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 100.0),
        ];

        let offset = offset_polyline(&square, 10.0, JoinType::Miter, 4.0, true);
        assert!(!offset.is_empty());
        // Bounds should expand beyond 0..100
        let min_x = offset.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let max_x = offset.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        assert!(min_x < 0.0);
        assert!(max_x > 100.0);
    }

    #[test]
    fn test_drop_shadow_generation() {
        let obj = json!({
            "type": "path",
            "points": [[10.0, 10.0], [50.0, 50.0]],
            "fill": {"color": [1.0, 0.0, 0.0]}
        });

        let shadow = create_drop_shadow_object(&obj, 5.0, 5.0, 0.5);
        let pts = &shadow["points"];
        assert_eq!(pts[0][0], 15.0);
        assert_eq!(pts[0][1], 15.0);
        assert_eq!(shadow["fill"]["opacity"], 0.5);
    }

    #[test]
    fn test_full_stylism_document() {
        let mut doc = json!({
            "version": 1,
            "objects": [
                {
                    "type": "path",
                    "points": [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]],
                    "closed": true,
                    "fill": {"color": [0.0, 0.5, 1.0]}
                }
            ]
        });

        let params = StylismParams {
            offset_distance: 2.0,
            join: JoinType::Miter,
            miter_limit: 4.0,
            shadow_dx: 4.0,
            shadow_dy: 4.0,
            shadow_blur: 5.0,
            shadow_opacity: 0.3,
            shadow_enabled: true,
            glow_radius: 0.0,
            glow_opacity: 0.0,
            glow_enabled: false,
        };

        apply_stylism(&mut doc, &params);
        let objects = doc["objects"].as_array().unwrap();
        assert_eq!(objects.len(), 2); // shadow + primary object
    }
}
