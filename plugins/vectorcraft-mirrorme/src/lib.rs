//! VectorCraft MirrorMe symmetry and reflection plugin (VectorCraft ABI v1).
//! Inspired by Astute Graphics MirrorMe.

use serde_json::{Value, json};
use std::f64::consts::PI;

pub const MANIFEST: &str = r#"{
  "id": "org.vectorcraft.community.mirrorme",
  "name": "Vector MirrorMe",
  "version": "1.0.0",
  "kind": "filter",
  "author": "ArtCraft Store community",
  "description": "Real-time symmetry and mirroring tool for VectorCraft: mirror selections across arbitrary axes, quad axes, or radial kaleidoscopic sectors.",
  "params": {
    "axis_angle": {"type": "number", "min": -180, "max": 180, "default": 90.0},
    "pivot_x": {"type": "number", "default": 0.0},
    "pivot_y": {"type": "number", "default": 0.0},
    "symmetry_axes": {"type": "integer", "min": 1, "max": 12, "default": 1},
    "keep_original": {"type": "boolean", "default": true}
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

#[derive(Debug, Clone, Copy)]
pub struct MirrorParams {
    pub axis_angle_deg: f64,
    pub pivot_x: f64,
    pub pivot_y: f64,
    pub symmetry_axes: usize,
    pub keep_original: bool,
}

impl MirrorParams {
    pub fn from_json(val: &Value) -> Self {
        Self {
            axis_angle_deg: val.get("axis_angle").and_then(|v| v.as_f64()).unwrap_or(90.0),
            pivot_x: val.get("pivot_x").and_then(|v| v.as_f64()).unwrap_or(0.0),
            pivot_y: val.get("pivot_y").and_then(|v| v.as_f64()).unwrap_or(0.0),
            symmetry_axes: val.get("symmetry_axes").and_then(|v| v.as_u64()).unwrap_or(1).clamp(1, 12) as usize,
            keep_original: val.get("keep_original").and_then(|v| v.as_bool()).unwrap_or(true),
        }
    }
}

/// Reflects a 2D coordinate (x, y) across an axis passing through (px, py) at angle theta_rad.
pub fn reflect_point(x: f64, y: f64, px: f64, py: f64, theta_rad: f64) -> (f64, f64) {
    let two_theta = 2.0 * theta_rad;
    let cos2 = two_theta.cos();
    let sin2 = two_theta.sin();
    let dx = x - px;
    let dy = y - py;

    let rx = px + dx * cos2 + dy * sin2;
    let ry = py + dx * sin2 - dy * cos2;

    (rx, ry)
}

/// Transforms an SVG path string (`M ... C ... L ... Z`) by reflecting all coordinate pairs.
pub fn reflect_svg_path(d: &str, px: f64, py: f64, theta_rad: f64) -> String {
    let mut out = String::with_capacity(d.len() + 16);
    let tokens = tokenize_svg_path(d);
    let mut i = 0;

    while i < tokens.len() {
        let tok = &tokens[i];
        if tok.len() == 1 && tok.chars().next().unwrap().is_ascii_alphabetic() {
            let cmd = tok.chars().next().unwrap();
            out.push(cmd);
            out.push(' ');
            i += 1;
        } else {
            // Coordinate pair (x, y)
            if let Ok(x) = tokens[i].parse::<f64>() {
                if i + 1 < tokens.len() {
                    if let Ok(y) = tokens[i + 1].parse::<f64>() {
                        let (rx, ry) = reflect_point(x, y, px, py, theta_rad);
                        out.push_str(&format!("{:.3} {:.3} ", rx, ry));
                        i += 2;
                        continue;
                    }
                }
                out.push_str(&format!("{} ", tokens[i]));
                i += 1;
            } else {
                out.push_str(&format!("{} ", tokens[i]));
                i += 1;
            }
        }
    }

    out.trim_end().to_string()
}

fn tokenize_svg_path(d: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in d.chars() {
        if ch.is_ascii_alphabetic() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            tokens.push(ch.to_string());
        } else if ch.is_whitespace() || ch == ',' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else if ch == '-' {
            if !current.is_empty() && !current.ends_with('e') && !current.ends_with('E') {
                tokens.push(current.clone());
                current.clear();
            }
            current.push(ch);
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Clones and reflects an object's geometry across the given axis.
pub fn mirror_single_object(obj: &Value, px: f64, py: f64, theta_rad: f64) -> Value {
    let mut mirrored = obj.clone();

    if let Value::Object(map) = &mut mirrored {
        // 1. Transform SVG path 'd' or 'path'
        if let Some(Value::String(d)) = map.get("d") {
            map.insert("d".to_string(), json!(reflect_svg_path(d, px, py, theta_rad)));
        }
        if let Some(Value::String(path)) = map.get("path") {
            map.insert("path".to_string(), json!(reflect_svg_path(path, px, py, theta_rad)));
        }

        // 2. Transform points array [[x, y], ...]
        if let Some(Value::Array(points)) = map.get_mut("points") {
            for pt in points {
                if let Value::Array(xy) = pt {
                    if xy.len() >= 2 {
                        let x = xy[0].as_f64().unwrap_or(0.0);
                        let y = xy[1].as_f64().unwrap_or(0.0);
                        let (rx, ry) = reflect_point(x, y, px, py, theta_rad);
                        xy[0] = json!(rx);
                        xy[1] = json!(ry);
                    }
                } else if let Value::Object(coord) = pt {
                    let x = coord.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let y = coord.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let (rx, ry) = reflect_point(x, y, px, py, theta_rad);
                    coord.insert("x".to_string(), json!(rx));
                    coord.insert("y".to_string(), json!(ry));
                }
            }
        }

        // 3. Transform explicit position x, y
        if let (Some(x_val), Some(y_val)) = (map.get("x"), map.get("y")) {
            let x = x_val.as_f64().unwrap_or(0.0);
            let y = y_val.as_f64().unwrap_or(0.0);
            let (rx, ry) = reflect_point(x, y, px, py, theta_rad);
            map.insert("x".to_string(), json!(rx));
            map.insert("y".to_string(), json!(ry));
        }

        // 4. Transform children recursively
        if let Some(Value::Array(children)) = map.get_mut("children") {
            let mut reflected_children = Vec::with_capacity(children.len());
            for child in children.iter() {
                reflected_children.push(mirror_single_object(child, px, py, theta_rad));
            }
            *children = reflected_children;
        }
    }

    mirrored
}

/// Applies MirrorMe symmetry replication across all objects in a document.
pub fn apply_mirrorme(doc: &mut Value, params: &MirrorParams) {
    if let Some(objects) = doc.get_mut("objects").and_then(|o| o.as_array_mut()) {
        let base_angle_rad = params.axis_angle_deg * PI / 180.0;
        let mut new_objects = Vec::new();

        for obj in objects.iter() {
            if params.keep_original {
                new_objects.push(obj.clone());
            }

            // For each symmetry axis line
            for k in 0..params.symmetry_axes {
                let axis_rad = base_angle_rad + (k as f64 * PI / params.symmetry_axes as f64);
                let mirrored = mirror_single_object(obj, params.pivot_x, params.pivot_y, axis_rad);
                new_objects.push(mirrored);
            }
        }

        *objects = new_objects;
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
    let p = MirrorParams::from_json(&params_val);

    apply_mirrorme(&mut doc, &p);

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
    fn test_vertical_reflection() {
        // Vertical axis (angle = 90 deg) at x = 100
        let (rx, ry) = reflect_point(50.0, 30.0, 100.0, 0.0, 90.0 * PI / 180.0);
        assert!((rx - 150.0).abs() < 1e-4);
        assert!((ry - 30.0).abs() < 1e-4);
    }

    #[test]
    fn test_horizontal_reflection() {
        // Horizontal axis (angle = 0 deg) at y = 50
        let (rx, ry) = reflect_point(20.0, 10.0, 0.0, 50.0, 0.0);
        assert!((rx - 20.0).abs() < 1e-4);
        assert!((ry - 90.0).abs() < 1e-4);
    }

    #[test]
    fn test_svg_path_reflection() {
        let path = "M 10 20 L 30 40 Z";
        // Vertical axis at x = 50
        let reflected = reflect_svg_path(path, 50.0, 0.0, 90.0 * PI / 180.0);
        assert!(reflected.contains("M 90.000 20.000"));
        assert!(reflected.contains("L 70.000 40.000"));
        assert!(reflected.ends_with('Z'));
    }

    #[test]
    fn test_doc_mirror_replication() {
        let mut doc = json!({
            "version": 1,
            "objects": [
                {
                    "type": "path",
                    "d": "M 10 10 L 20 20",
                    "points": [[10.0, 10.0], [20.0, 20.0]]
                }
            ]
        });

        let params = MirrorParams {
            axis_angle_deg: 90.0,
            pivot_x: 0.0,
            pivot_y: 0.0,
            symmetry_axes: 1,
            keep_original: true,
        };

        apply_mirrorme(&mut doc, &params);

        let objects = doc["objects"].as_array().unwrap();
        assert_eq!(objects.len(), 2); // original + 1 mirrored
        let p_mirrored = &objects[1]["points"];
        assert!((p_mirrored[0][0].as_f64().unwrap() - (-10.0)).abs() < 1e-4);
        assert!((p_mirrored[1][0].as_f64().unwrap() - (-20.0)).abs() < 1e-4);
    }
}
