//! VectorCraft ColliderScribe Super Marquee selection and query plugin (VectorCraft ABI v1).
//! Inspired by Astute Graphics ColliderScribe.

use serde_json::{Value, json};

pub const MANIFEST: &str = r#"{
  "id": "org.vectorcraft.community.colliderscribe",
  "name": "Vector ColliderScribe",
  "version": "1.0.0",
  "kind": "filter",
  "author": "ArtCraft Store community",
  "description": "Super Marquee selection and geometric query tool for VectorCraft: rectangular/elliptical marquees, enclosed artwork detection, alternating and random selections.",
  "params": {
    "scope": {"type": "choice", "options": ["selection", "marquee"], "default": "selection"},
    "shape": {"type": "choice", "options": ["rectangle", "ellipse"], "default": "rectangle"},
    "x": {"type": "number", "min": -10000.0, "max": 10000.0, "default": 0.0},
    "y": {"type": "number", "min": -10000.0, "max": 10000.0, "default": 0.0},
    "width": {"type": "number", "min": 0.0, "max": 10000.0, "default": 1000.0},
    "height": {"type": "number", "min": 0.0, "max": 10000.0, "default": 1000.0},
    "mode": {"type": "choice", "options": ["enclosed", "intersecting"], "default": "enclosed"},
    "filter": {"type": "choice", "options": ["alternate", "random", "all"], "default": "alternate"},
    "alternate_step": {"type": "int", "min": 1, "max": 20, "default": 2},
    "random_percent": {"type": "number", "min": 0.0, "max": 100.0, "default": 50.0},
    "seed": {"type": "int", "min": 0, "max": 2147483647, "default": 42},
    "action": {"type": "choice", "options": ["isolate", "exclude", "color_tag"], "default": "isolate"}
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

pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 0x853c49e6748fea9b } else { seed } }
    }

    pub fn next_f64(&mut self) -> f64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        let v = x.wrapping_mul(0x2545F4914F6CDD1D);
        (v >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryScope {
    Selection,
    Marquee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarqueeShape {
    Rectangle,
    Ellipse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnclosureMode {
    Enclosed,
    Intersecting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionFilter {
    All,
    Alternate,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionAction {
    Isolate,
    Exclude,
    ColorTag,
}

#[derive(Debug, Clone)]
pub struct ColliderParams {
    pub scope: QueryScope,
    pub shape: MarqueeShape,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub mode: EnclosureMode,
    pub filter: SelectionFilter,
    pub alternate_step: usize,
    pub random_percent: f64,
    pub seed: u64,
    pub action: SelectionAction,
}

impl ColliderParams {
    pub fn from_json(val: &Value) -> Self {
        let scope = match val.get("scope").and_then(|v| v.as_str()).unwrap_or("selection") {
            "marquee" => QueryScope::Marquee,
            _ => QueryScope::Selection,
        };
        let shape = match val.get("shape").and_then(|v| v.as_str()).unwrap_or("rectangle") {
            "ellipse" => MarqueeShape::Ellipse,
            _ => MarqueeShape::Rectangle,
        };
        let mode = match val.get("mode").and_then(|v| v.as_str()).unwrap_or("enclosed") {
            "intersecting" => EnclosureMode::Intersecting,
            _ => EnclosureMode::Enclosed,
        };
        let filter = match val.get("filter").and_then(|v| v.as_str()).unwrap_or("alternate") {
            "all" => SelectionFilter::All,
            "random" => SelectionFilter::Random,
            _ => SelectionFilter::Alternate,
        };
        let action = match val.get("action").and_then(|v| v.as_str()).unwrap_or("isolate") {
            "exclude" => SelectionAction::Exclude,
            "color_tag" => SelectionAction::ColorTag,
            _ => SelectionAction::Isolate,
        };

        Self {
            scope,
            shape,
            x: val.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
            y: val.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
            width: val.get("width").and_then(|v| v.as_f64()).unwrap_or(1000.0).max(0.0),
            height: val.get("height").and_then(|v| v.as_f64()).unwrap_or(1000.0).max(0.0),
            mode,
            filter,
            alternate_step: val.get("alternate_step").and_then(|v| v.as_u64()).unwrap_or(2).max(1) as usize,
            random_percent: val.get("random_percent").and_then(|v| v.as_f64()).unwrap_or(50.0).clamp(0.0, 100.0),
            seed: val.get("seed").and_then(|v| v.as_u64()).unwrap_or(42),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

pub fn extract_object_points(obj: &Value) -> Vec<(f64, f64)> {
    let mut out = Vec::new();

    let subpaths_opt = obj.get("path")
        .and_then(|p| p.get("subpaths"))
        .or_else(|| obj.get("kind").and_then(|k| k.get("path")).and_then(|p| p.get("subpaths")))
        .and_then(|s| s.as_array());

    if let Some(subpaths) = subpaths_opt {
        for subpath in subpaths {
            if let Some(anchors) = subpath.get("anchors").and_then(|a| a.as_array()) {
                for anc in anchors {
                    if let Some(p) = anc.get("p").and_then(|v| v.as_array()) {
                        if p.len() >= 2 {
                            let x = p[0].as_f64().unwrap_or(0.0);
                            let y = p[1].as_f64().unwrap_or(0.0);
                            out.push((x, y));
                        }
                    }
                }
            }
        }
    }

    if let Some(pts) = obj.get("points").and_then(|p| p.as_array()) {
        for pt in pts {
            if let Value::Array(xy) = pt {
                if xy.len() >= 2 {
                    let x = xy[0].as_f64().unwrap_or(0.0);
                    let y = xy[1].as_f64().unwrap_or(0.0);
                    out.push((x, y));
                }
            } else if let Value::Object(coord) = pt {
                let x = coord.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let y = coord.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                out.push((x, y));
            }
        }
    }
    if out.is_empty() {
        let x = obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let y = obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let w = obj.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let h = obj.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0);
        out.push((x, y));
        if w > 0.0 || h > 0.0 {
            out.push((x + w, y));
            out.push((x + w, y + h));
            out.push((x, y + h));
        }
    }
    out
}

pub fn compute_bounds(points: &[(f64, f64)]) -> Option<BoundingBox> {
    if points.is_empty() {
        return None;
    }
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for &(x, y) in points {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    Some(BoundingBox { min_x, min_y, max_x, max_y })
}

/// Checks if an object satisfies the geometric marquee query
pub fn matches_marquee(points: &[(f64, f64)], params: &ColliderParams) -> bool {
    if params.scope == QueryScope::Selection {
        return true;
    }

    let bounds = match compute_bounds(points) {
        Some(b) => b,
        None => return false,
    };

    match params.shape {
        MarqueeShape::Rectangle => {
            let mar_min_x = params.x;
            let mar_min_y = params.y;
            let mar_max_x = params.x + params.width;
            let mar_max_y = params.y + params.height;

            match params.mode {
                EnclosureMode::Enclosed => {
                    bounds.min_x >= mar_min_x
                        && bounds.max_x <= mar_max_x
                        && bounds.min_y >= mar_min_y
                        && bounds.max_y <= mar_max_y
                }
                EnclosureMode::Intersecting => {
                    bounds.max_x >= mar_min_x
                        && bounds.min_x <= mar_max_x
                        && bounds.max_y >= mar_min_y
                        && bounds.min_y <= mar_max_y
                }
            }
        }
        MarqueeShape::Ellipse => {
            let cx = params.x + params.width * 0.5;
            let cy = params.y + params.height * 0.5;
            let rx = (params.width * 0.5).max(1e-4);
            let ry = (params.height * 0.5).max(1e-4);

            let pt_inside = |px: f64, py: f64| -> bool {
                let nx = (px - cx) / rx;
                let ny = (py - cy) / ry;
                nx * nx + ny * ny <= 1.0
            };

            match params.mode {
                EnclosureMode::Enclosed => {
                    points.iter().all(|&(x, y)| pt_inside(x, y))
                }
                EnclosureMode::Intersecting => {
                    points.iter().any(|&(x, y)| pt_inside(x, y))
                }
            }
        }
    }
}

fn apply_color_tag(obj: &mut Value) {
    if let Value::Object(map) = obj {
        let tag_paint = json!({
            "type": "solid",
            "color": {
                "model": "rgb",
                "r": 0.0,
                "g": 0.9,
                "b": 1.0
            }
        });
        let tag_item = json!({
            "kind": "fill",
            "paint": tag_paint.clone()
        });
        map.insert("appearance".to_string(), json!({ "items": [tag_item] }));
        map.insert("fills".to_string(), json!([tag_paint.clone()]));
        map.insert("fill".to_string(), tag_paint);
    }
}

pub fn apply_colliderscribe(doc: &mut Value, params: &ColliderParams) {
    if let Some(objects) = doc.get_mut("objects").and_then(|o| o.as_array_mut()) {
        let mut final_objects = Vec::new();
        let mut rng = SimpleRng::new(params.seed);
        let mut match_counter = 0;

        for mut obj in objects.drain(..) {
            let pts = extract_object_points(&obj);
            let in_marquee = matches_marquee(&pts, params);

            let is_matched = if in_marquee {
                match params.filter {
                    SelectionFilter::All => true,
                    SelectionFilter::Alternate => {
                        let selected = (match_counter % params.alternate_step) == 0;
                        match_counter += 1;
                        selected
                    }
                    SelectionFilter::Random => {
                        let r = rng.next_f64() * 100.0;
                        r <= params.random_percent
                    }
                }
            } else {
                false
            };

            match params.action {
                SelectionAction::Isolate => {
                    if is_matched {
                        final_objects.push(obj);
                    }
                }
                SelectionAction::Exclude => {
                    if !is_matched {
                        final_objects.push(obj);
                    }
                }
                SelectionAction::ColorTag => {
                    if is_matched {
                        apply_color_tag(&mut obj);
                    }
                    final_objects.push(obj);
                }
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
    let p = ColliderParams::from_json(&params_val);

    apply_colliderscribe(&mut doc, &p);

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
    fn test_alternate_selection() {
        let mut doc = json!({
            "objects": [
                {"id": 1, "points": [[10.0, 10.0]]},
                {"id": 2, "points": [[20.0, 20.0]]},
                {"id": 3, "points": [[30.0, 30.0]]},
                {"id": 4, "points": [[40.0, 40.0]]},
            ]
        });

        let params = ColliderParams {
            scope: QueryScope::Selection,
            shape: MarqueeShape::Rectangle,
            x: 0.0,
            y: 0.0,
            width: 1000.0,
            height: 1000.0,
            mode: EnclosureMode::Enclosed,
            filter: SelectionFilter::Alternate,
            alternate_step: 2,
            random_percent: 50.0,
            seed: 42,
            action: SelectionAction::Isolate,
        };

        apply_colliderscribe(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        assert_eq!(objs.len(), 2);
        assert_eq!(objs[0]["id"], 1);
        assert_eq!(objs[1]["id"], 3);
    }

    #[test]
    fn test_marquee_enclosure_with_kind_path() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "kind": {
                        "type": "path",
                        "path": {
                            "subpaths": [
                                {
                                    "anchors": [{"p": [50.0, 50.0]}]
                                }
                            ]
                        }
                    }
                },
                {
                    "id": 2,
                    "kind": {
                        "type": "path",
                        "path": {
                            "subpaths": [
                                {
                                    "anchors": [{"p": [500.0, 500.0]}]
                                }
                            ]
                        }
                    }
                }
            ]
        });

        let params = ColliderParams {
            scope: QueryScope::Marquee,
            shape: MarqueeShape::Rectangle,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            mode: EnclosureMode::Enclosed,
            filter: SelectionFilter::All,
            alternate_step: 1,
            random_percent: 100.0,
            seed: 42,
            action: SelectionAction::Isolate,
        };

        apply_colliderscribe(&mut doc, &params);
        let objs = doc["objects"].as_array().unwrap();
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0]["id"], 1);
    }
}
