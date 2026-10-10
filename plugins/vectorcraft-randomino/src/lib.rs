//! VectorCraft Randomino generative variation plugin (VectorCraft ABI v1).
//! Inspired by Astute Graphics Randomino.

use serde_json::{Value, json};
use std::f64::consts::PI;

pub const MANIFEST: &str = r#"{
  "id": "org.vectorcraft.community.randomino",
  "name": "Vector Randomino",
  "version": "1.0.0",
  "kind": "filter",
  "author": "ArtCraft Store community",
  "description": "Generative variation tool for VectorCraft: randomize position, rotation, scale, color, opacity, and stacking order with deterministic seed control.",
  "params": {
    "seed": {"type": "int", "min": 0, "max": 2147483647, "default": 42},
    "pos_x_jitter": {"type": "number", "min": 0.0, "max": 200.0, "default": 0.0},
    "pos_y_jitter": {"type": "number", "min": 0.0, "max": 200.0, "default": 0.0},
    "rot_min": {"type": "number", "min": -180.0, "max": 180.0, "default": 0.0},
    "rot_max": {"type": "number", "min": -180.0, "max": 180.0, "default": 0.0},
    "scale_min": {"type": "number", "min": 0.1, "max": 5.0, "default": 1.0},
    "scale_max": {"type": "number", "min": 0.1, "max": 5.0, "default": 1.0},
    "uniform_scale": {"type": "bool", "default": true},
    "hue_jitter": {"type": "number", "min": 0.0, "max": 180.0, "default": 0.0},
    "sat_jitter": {"type": "number", "min": 0.0, "max": 100.0, "default": 0.0},
    "lightness_jitter": {"type": "number", "min": 0.0, "max": 100.0, "default": 0.0},
    "opacity_min": {"type": "number", "min": 0.0, "max": 1.0, "default": 1.0},
    "opacity_max": {"type": "number", "min": 0.0, "max": 1.0, "default": 1.0},
    "shuffle_stack": {"type": "bool", "default": false}
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

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 0x853c49e6748fea9b } else { seed } }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn range(&mut self, min: f64, max: f64) -> f64 {
        if min >= max { min } else { min + self.next_f64() * (max - min) }
    }
}

#[derive(Debug, Clone)]
pub struct RandominoParams {
    pub seed: u64,
    pub pos_x_jitter: f64,
    pub pos_y_jitter: f64,
    pub rot_min: f64,
    pub rot_max: f64,
    pub scale_min: f64,
    pub scale_max: f64,
    pub uniform_scale: bool,
    pub hue_jitter: f64,
    pub sat_jitter: f64,
    pub lightness_jitter: f64,
    pub opacity_min: f64,
    pub opacity_max: f64,
    pub shuffle_stack: bool,
}

impl RandominoParams {
    pub fn from_json(val: &Value) -> Self {
        Self {
            seed: val.get("seed").and_then(|v| v.as_u64()).unwrap_or(42),
            pos_x_jitter: val.get("pos_x_jitter").and_then(|v| v.as_f64()).unwrap_or(0.0),
            pos_y_jitter: val.get("pos_y_jitter").and_then(|v| v.as_f64()).unwrap_or(0.0),
            rot_min: val.get("rot_min").and_then(|v| v.as_f64()).unwrap_or(0.0),
            rot_max: val.get("rot_max").and_then(|v| v.as_f64()).unwrap_or(0.0),
            scale_min: val.get("scale_min").and_then(|v| v.as_f64()).unwrap_or(1.0),
            scale_max: val.get("scale_max").and_then(|v| v.as_f64()).unwrap_or(1.0),
            uniform_scale: val.get("uniform_scale").and_then(|v| v.as_bool()).unwrap_or(true),
            hue_jitter: val.get("hue_jitter").and_then(|v| v.as_f64()).unwrap_or(0.0),
            sat_jitter: val.get("sat_jitter").and_then(|v| v.as_f64()).unwrap_or(0.0),
            lightness_jitter: val.get("lightness_jitter").and_then(|v| v.as_f64()).unwrap_or(0.0),
            opacity_min: val.get("opacity_min").and_then(|v| v.as_f64()).unwrap_or(1.0),
            opacity_max: val.get("opacity_max").and_then(|v| v.as_f64()).unwrap_or(1.0),
            shuffle_stack: val.get("shuffle_stack").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

pub fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let l = (max + min) / 2.0;

    if delta.abs() < 1e-6 {
        return (0.0, 0.0, l);
    }

    let s = if l > 0.5 { delta / (2.0 - max - min) } else { delta / (max + min) };

    let mut h = if (max - r).abs() < 1e-6 {
        (g - b) / delta + (if g < b { 6.0 } else { 0.0 })
    } else if (max - g).abs() < 1e-6 {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    h *= 60.0;
    (h, s, l)
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    if s.abs() < 1e-6 {
        return (l, l, l);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let h_norm = (h % 360.0 + 360.0) % 360.0 / 360.0;

    let r = hue_to_rgb(p, q, h_norm + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h_norm);
    let b = hue_to_rgb(p, q, h_norm - 1.0 / 3.0);
    (r, g, b)
}

fn jitter_color(val: &mut Value, dh: f64, ds: f64, dl: f64) {
    match val {
        Value::Object(map) => {
            if let Some(model) = map.get("model").and_then(|m| m.as_str()) {
                if model == "rgb" {
                    let r = map.get("r").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let g = map.get("g").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let b = map.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let (h, s, l) = rgb_to_hsl(r, g, b);
                    let nh = (h + dh) % 360.0;
                    let ns = (s + ds / 100.0).clamp(0.0, 1.0);
                    let nl = (l + dl / 100.0).clamp(0.0, 1.0);
                    let (nr, ng, nb) = hsl_to_rgb(nh, ns, nl);
                    map.insert("r".to_string(), json!(nr));
                    map.insert("g".to_string(), json!(ng));
                    map.insert("b".to_string(), json!(nb));
                }
            } else if let Some(c) = map.get_mut("color") {
                jitter_color(c, dh, ds, dl);
            }
        }
        Value::Array(arr) => {
            if arr.len() >= 3 {
                let r = arr[0].as_f64().unwrap_or(0.0).clamp(0.0, 1.0);
                let g = arr[1].as_f64().unwrap_or(0.0).clamp(0.0, 1.0);
                let b = arr[2].as_f64().unwrap_or(0.0).clamp(0.0, 1.0);

                let (h, s, l) = rgb_to_hsl(r, g, b);
                let nh = (h + dh) % 360.0;
                let ns = (s + ds / 100.0).clamp(0.0, 1.0);
                let nl = (l + dl / 100.0).clamp(0.0, 1.0);

                let (nr, ng, nb) = hsl_to_rgb(nh, ns, nl);
                arr[0] = json!(nr);
                arr[1] = json!(ng);
                arr[2] = json!(nb);
            }
        }
        _ => {}
    }
}

/// Computes the bounding box center (cx, cy) of an object
fn get_object_centroid(obj: &Value) -> (f64, f64) {
    if let Some(path_val) = obj.get("path") {
        if let Some(subpaths) = path_val.get("subpaths").and_then(|s| s.as_array()) {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut count = 0;
            for subpath in subpaths {
                if let Some(anchors) = subpath.get("anchors").and_then(|a| a.as_array()) {
                    for anc in anchors {
                        if let Some(p) = anc.get("p").and_then(|v| v.as_array()) {
                            if p.len() >= 2 {
                                sum_x += p[0].as_f64().unwrap_or(0.0);
                                sum_y += p[1].as_f64().unwrap_or(0.0);
                                count += 1;
                            }
                        }
                    }
                }
            }
            if count > 0 {
                return (sum_x / count as f64, sum_y / count as f64);
            }
        }
    }
    if let Some(pts) = obj.get("points").and_then(|p| p.as_array()) {
        if !pts.is_empty() {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut count = 0;
            for pt in pts {
                if let Value::Array(xy) = pt {
                    if xy.len() >= 2 {
                        sum_x += xy[0].as_f64().unwrap_or(0.0);
                        sum_y += xy[1].as_f64().unwrap_or(0.0);
                        count += 1;
                    }
                }
            }
            if count > 0 {
                return (sum_x / count as f64, sum_y / count as f64);
            }
        }
    }
    let x = obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y = obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
    (x, y)
}

pub fn transform_single_object(obj: &mut Value, params: &RandominoParams, rng: &mut Rng) {
    let (cx, cy) = get_object_centroid(obj);

    // 1. Jitter values
    let dx = if params.pos_x_jitter > 0.0 { rng.range(-params.pos_x_jitter, params.pos_x_jitter) } else { 0.0 };
    let dy = if params.pos_y_jitter > 0.0 { rng.range(-params.pos_y_jitter, params.pos_y_jitter) } else { 0.0 };
    let deg = if params.rot_min != params.rot_max { rng.range(params.rot_min, params.rot_max) } else { params.rot_min };
    let rad = deg * PI / 180.0;
    let cos_t = rad.cos();
    let sin_t = rad.sin();

    let sx = if params.scale_min != params.scale_max { rng.range(params.scale_min, params.scale_max) } else { params.scale_min };
    let sy = if params.uniform_scale {
        sx
    } else if params.scale_min != params.scale_max {
        rng.range(params.scale_min, params.scale_max)
    } else {
        params.scale_min
    };

    let transform_xy = |px: f64, py: f64| -> (f64, f64) {
        let scaled_x = (px - cx) * sx;
        let scaled_y = (py - cy) * sy;
        let rot_x = scaled_x * cos_t - scaled_y * sin_t;
        let rot_y = scaled_x * sin_t + scaled_y * cos_t;
        (rot_x + cx + dx, rot_y + cy + dy)
    };

    // 2. Transform points & subpath anchors
    if let Value::Object(map) = obj {
        if let Some(path_val) = map.get_mut("path") {
            if let Some(subpaths) = path_val.get_mut("subpaths").and_then(|s| s.as_array_mut()) {
                for subpath in subpaths {
                    if let Some(anchors) = subpath.get_mut("anchors").and_then(|a| a.as_array_mut()) {
                        for anchor in anchors {
                            for key in &["p", "in", "out"] {
                                if let Some(Value::Array(arr)) = anchor.get_mut(*key) {
                                    if arr.len() >= 2 {
                                        let px = arr[0].as_f64().unwrap_or(0.0);
                                        let py = arr[1].as_f64().unwrap_or(0.0);
                                        let (rx, ry) = transform_xy(px, py);
                                        arr[0] = json!(rx);
                                        arr[1] = json!(ry);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(pts) = map.get_mut("points").and_then(|p| p.as_array_mut()) {
            for pt in pts {
                if let Value::Array(xy) = pt {
                    if xy.len() >= 2 {
                        let px = xy[0].as_f64().unwrap_or(0.0);
                        let py = xy[1].as_f64().unwrap_or(0.0);
                        let (rx, ry) = transform_xy(px, py);
                        xy[0] = json!(rx);
                        xy[1] = json!(ry);
                    }
                }
            }
        }

        // Transform x, y
        let new_x = map.get("x").and_then(|v| v.as_f64()).map(|x| (x - cx) * sx * cos_t - (0.0) * sin_t + cx + dx);
        let new_y = map.get("y").and_then(|v| v.as_f64()).map(|y| (0.0) * sin_t + (y - cy) * sy * cos_t + cy + dy);
        if let (Some(nx), Some(ny)) = (new_x, new_y) {
            map.insert("x".to_string(), json!(nx));
            map.insert("y".to_string(), json!(ny));
        }

        // 3. Color jitter
        if params.hue_jitter > 0.0 || params.sat_jitter > 0.0 || params.lightness_jitter > 0.0 {
            let dh = if params.hue_jitter > 0.0 { rng.range(-params.hue_jitter, params.hue_jitter) } else { 0.0 };
            let ds = if params.sat_jitter > 0.0 { rng.range(-params.sat_jitter, params.sat_jitter) } else { 0.0 };
            let dl = if params.lightness_jitter > 0.0 { rng.range(-params.lightness_jitter, params.lightness_jitter) } else { 0.0 };

            if let Some(fills) = map.get_mut("fills").and_then(|f| f.as_array_mut()) {
                for fill in fills {
                    if let Some(c) = fill.get_mut("color") {
                        jitter_color(c, dh, ds, dl);
                    }
                }
            }
            if let Some(fill) = map.get_mut("fill") {
                if let Some(c) = fill.get_mut("color") {
                    jitter_color(c, dh, ds, dl);
                }
            }
            if let Some(strokes) = map.get_mut("strokes").and_then(|s| s.as_array_mut()) {
                for stroke in strokes {
                    if let Some(paint) = stroke.get_mut("paint") {
                        if let Some(c) = paint.get_mut("color") {
                            jitter_color(c, dh, ds, dl);
                        }
                    }
                }
            }
            if let Some(stroke) = map.get_mut("stroke") {
                if let Some(c) = stroke.get_mut("color") {
                    jitter_color(c, dh, ds, dl);
                }
            }
        }

        // 4. Opacity jitter
        if params.opacity_min != 1.0 || params.opacity_max != 1.0 {
            let op = rng.range(params.opacity_min, params.opacity_max);
            map.insert("opacity".to_string(), json!(op));
        }
    }
}

pub fn apply_randomino(doc: &mut Value, params: &RandominoParams) {
    let mut rng = Rng::new(params.seed);

    if let Some(objects) = doc.get_mut("objects").and_then(|o| o.as_array_mut()) {
        // 1. Transform each object
        for obj in objects.iter_mut() {
            transform_single_object(obj, params, &mut rng);
        }

        // 2. Optional Stacking Order Shuffle (Fisher-Yates)
        if params.shuffle_stack && objects.len() > 1 {
            let n = objects.len();
            for i in (1..n).rev() {
                let j = (rng.next_u64() as usize) % (i + 1);
                objects.swap(i, j);
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
    let p = RandominoParams::from_json(&params_val);

    apply_randomino(&mut doc, &p);

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
    fn test_deterministic_seed() {
        let mut doc1 = json!({
            "objects": [{"points": [[0.0, 0.0], [10.0, 10.0]]}]
        });
        let mut doc2 = doc1.clone();

        let params = RandominoParams {
            seed: 12345,
            pos_x_jitter: 15.0,
            pos_y_jitter: 15.0,
            rot_min: -30.0,
            rot_max: 30.0,
            scale_min: 0.8,
            scale_max: 1.2,
            uniform_scale: true,
            hue_jitter: 10.0,
            sat_jitter: 10.0,
            lightness_jitter: 10.0,
            opacity_min: 0.5,
            opacity_max: 1.0,
            shuffle_stack: false,
        };

        apply_randomino(&mut doc1, &params);
        apply_randomino(&mut doc2, &params);

        assert_eq!(doc1, doc2);
    }

    #[test]
    fn test_position_jitter_bounds() {
        let mut doc = json!({
            "objects": [{"points": [[50.0, 50.0]]}]
        });
        let params = RandominoParams {
            seed: 99,
            pos_x_jitter: 10.0,
            pos_y_jitter: 10.0,
            rot_min: 0.0,
            rot_max: 0.0,
            scale_min: 1.0,
            scale_max: 1.0,
            uniform_scale: true,
            hue_jitter: 0.0,
            sat_jitter: 0.0,
            lightness_jitter: 0.0,
            opacity_min: 1.0,
            opacity_max: 1.0,
            shuffle_stack: false,
        };

        apply_randomino(&mut doc, &params);
        let pt = &doc["objects"][0]["points"][0];
        let x = pt[0].as_f64().unwrap();
        let y = pt[1].as_f64().unwrap();
        assert!(x >= 40.0 && x <= 60.0);
        assert!(y >= 40.0 && y <= 60.0);
    }

    #[test]
    fn test_shuffle_stack() {
        let mut doc = json!({
            "objects": [
                {"id": 1, "points": [[0.0, 0.0]]},
                {"id": 2, "points": [[1.0, 1.0]]},
                {"id": 3, "points": [[2.0, 2.0]]},
                {"id": 4, "points": [[3.0, 3.0]]},
            ]
        });
        let params = RandominoParams {
            seed: 777,
            pos_x_jitter: 0.0,
            pos_y_jitter: 0.0,
            rot_min: 0.0,
            rot_max: 0.0,
            scale_min: 1.0,
            scale_max: 1.0,
            uniform_scale: true,
            hue_jitter: 0.0,
            sat_jitter: 0.0,
            lightness_jitter: 0.0,
            opacity_min: 1.0,
            opacity_max: 1.0,
            shuffle_stack: true,
        };

        apply_randomino(&mut doc, &params);
        let ids: Vec<i64> = doc["objects"].as_array().unwrap().iter().map(|o| o["id"].as_i64().unwrap()).collect();
        // With seed 777, order should be shuffled
        assert_ne!(ids, vec![1, 2, 3, 4]);
    }
}
