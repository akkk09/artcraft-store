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
    "shape": {"type": "choice", "options": ["rectangle", "ellipse"], "default": "rectangle"},
    "x": {"type": "number", "min": -10000.0, "max": 10000.0, "default": 0.0},
    "y": {"type": "number", "min": -10000.0, "max": 10000.0, "default": 0.0},
    "width": {"type": "number", "min": 0.0, "max": 10000.0, "default": 100.0},
    "height": {"type": "number", "min": 0.0, "max": 10000.0, "default": 100.0},
    "mode": {"type": "choice", "options": ["enclosed", "intersecting"], "default": "enclosed"},
    "filter": {"type": "choice", "options": ["all", "alternate", "random"], "default": "all"},
    "alternate_step": {"type": "int", "min": 1, "max": 20, "default": 2},
    "random_percent": {"type": "number", "min": 0.0, "max": 100.0, "default": 50.0},
    "seed": {"type": "int", "min": 0, "max": 2147483647, "default": 42},
    "action": {"type": "choice", "options": ["mark_selected", "isolate", "exclude"], "default": "mark_selected"}
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
    MarkSelected,
    Isolate,
    Exclude,
}

#[derive(Debug, Clone)]
pub struct ColliderParams {
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
        let shape = match val.get("shape").and_then(|v| v.as_str()).unwrap_or("rectangle") {
            "ellipse" => MarqueeShape::Ellipse,
            _ => MarqueeShape::Rectangle,
        };
        let mode = match val.get("mode").and_then(|v| v.as_str()).unwrap_or("enclosed") {
            "intersecting" => EnclosureMode::Intersecting,
            _ => EnclosureMode::Enclosed,
        };
        let filter = match val.get("filter").and_then(|v| v.as_str()).unwrap_or("all") {
            "alternate" => SelectionFilter::Alternate,
            "random" => SelectionFilter::Random,
            _ => SelectionFilter::All,
        };
        let action = match val.get("action").and_then(|v| v.as_str()).unwrap_or("mark_selected") {
            "isolate" => SelectionAction::Isolate,
            "exclude" => SelectionAction::Exclude,
            _ => SelectionAction::MarkSelected,
        };

        Self {
            shape,
            x: val.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
            y: val.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
            width: val.get("width").and_then(|v| v.as_f64()).unwrap_or(100.0).max(0.0),
            height: val.get("height").and_then(|v| v.as_f64()).unwrap_or(100.0).max(0.0),
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

impl BoundingBox {
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.min_x && px <= self.max_x && py >= self.min_y && py <= self.max_y
    }
}

pub fn extract_object_points(obj: &Value) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    if let Some(path_val) = obj.get("path") {
        if let Some(subpaths) = path_val.get("subpaths").and_then(|s| s.as_array()) {
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
            let rx = (params.width * 0.5).max(1e-6);
            let ry = (params.height * 0.5).max(1e-6);

            let is_inside_ellipse = |x: f64, y: f64| -> bool {
                let dx = (x - cx) / rx;
                let dy = (y - cy) / ry;
                (dx * dx + dy * dy) <= 1.0 + 1e-4
            };

            match params.mode {
                EnclosureMode::Enclosed => {
                    // All vertices must lie within ellipse
                    points.iter().all(|&(x, y)| is_inside_ellipse(x, y))
                }
                EnclosureMode::Intersecting => {
                    // Either any vertex is inside ellipse, or center is within bounds
                    points.iter().any(|&(x, y)| is_inside_ellipse(x, y))
                        || bounds.contains_point(cx, cy)
                }
            }
        }
    }
}

pub fn apply_colliderscribe(doc: &mut Value, params: &ColliderParams) {
    let mut rng = SimpleRng::new(params.seed);
    let mut match_counter = 0;

    if let Some(objects) = doc.get_mut("objects").and_then(|o| o.as_array_mut()) {
        let mut final_objects = Vec::new();

        for obj in objects.drain(..) {
            let points = extract_object_points(&obj);
            let geom_match = matches_marquee(&points, params);

            let is_selected = if geom_match {
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
                SelectionAction::MarkSelected => {
                    let mut modified = obj;
                    if let Value::Object(map) = &mut modified {
                        map.insert("selected".to_string(), json!(is_selected));
                    }
                    final_objects.push(modified);
                }
                SelectionAction::Isolate => {
                    if is_selected {
                        final_objects.push(obj);
                    }
                }
                SelectionAction::Exclude => {
                    if !is_selected {
                        final_objects.push(obj);
                    }
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
    fn test_rect_enclosed_vs_intersecting() {
        let inside_pts = vec![(20.0, 20.0), (30.0, 30.0)];
        let straddle_pts = vec![(80.0, 80.0), (120.0, 120.0)];

        let mut params = ColliderParams {
            shape: MarqueeShape::Rectangle,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            mode: EnclosureMode::Enclosed,
            filter: SelectionFilter::All,
            alternate_step: 2,
            random_percent: 50.0,
            seed: 42,
            action: SelectionAction::MarkSelected,
        };

        // Enclosed mode: inside is true, straddle is false
        assert!(matches_marquee(&inside_pts, &params));
        assert!(!matches_marquee(&straddle_pts, &params));

        // Intersecting mode: straddle is true
        params.mode = EnclosureMode::Intersecting;
        assert!(matches_marquee(&straddle_pts, &params));
    }

    #[test]
    fn test_ellipse_marquee() {
        let center_pts = vec![(50.0, 50.0)];
        let corner_pts = vec![(5.0, 5.0)]; // Corner of 100x100 box is outside inscribed ellipse

        let params = ColliderParams {
            shape: MarqueeShape::Ellipse,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            mode: EnclosureMode::Enclosed,
            filter: SelectionFilter::All,
            alternate_step: 2,
            random_percent: 50.0,
            seed: 42,
            action: SelectionAction::MarkSelected,
        };

        assert!(matches_marquee(&center_pts, &params));
        assert!(!matches_marquee(&corner_pts, &params));
    }

    #[test]
    fn test_isolate_action() {
        let mut doc = json!({
            "objects": [
                {"id": 1, "points": [[10.0, 10.0]]}, // inside
                {"id": 2, "points": [[200.0, 200.0]]}, // outside
            ]
        });

        let params = ColliderParams {
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
