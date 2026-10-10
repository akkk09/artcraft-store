//! VectorCraft Recolor & Tone object filter plugin (VectorCraft ABI v1).

use serde_json::{Value, json};

pub const MANIFEST: &str = r#"{
  "id": "org.vectorcraft.community.recolor",
  "name": "Vector Recolor & Tone",
  "version": "1.0.0",
  "kind": "filter",
  "author": "ArtCraft Store community",
  "description": "Adjusts color tint, luminance and saturation across selected vector paths.",
  "params": {
    "saturation": {"type": "number", "min": 0, "max": 200, "default": 100},
    "brightness": {"type": "number", "min": -100, "max": 100, "default": 0}
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

fn adjust_channel(val: f64, sat: f64, bri: f64, avg: f64) -> f64 {
    let s = (val - avg) * (sat / 100.0) + avg;
    (s + (bri / 100.0)).clamp(0.0, 1.0)
}

pub fn adjust_color_val(color: &mut Value, sat: f64, bri: f64) {
    match color {
        Value::Object(map) => {
            let (r, g, b) = if map.get("model").and_then(|m| m.as_str()) == Some("rgb") || map.contains_key("r") {
                let r = map.get("r").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let g = map.get("g").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = map.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                (r, g, b)
            } else {
                return;
            };

            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            map.insert("r".to_string(), json!(adjust_channel(r, sat, bri, lum)));
            map.insert("g".to_string(), json!(adjust_channel(g, sat, bri, lum)));
            map.insert("b".to_string(), json!(adjust_channel(b, sat, bri, lum)));
        }
        Value::Array(arr) => {
            if arr.len() >= 3 {
                let r = arr[0].as_f64().unwrap_or(0.0);
                let g = arr[1].as_f64().unwrap_or(0.0);
                let b = arr[2].as_f64().unwrap_or(0.0);
                let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                arr[0] = json!(adjust_channel(r, sat, bri, lum));
                arr[1] = json!(adjust_channel(g, sat, bri, lum));
                arr[2] = json!(adjust_channel(b, sat, bri, lum));
            }
        }
        _ => {}
    }
}

fn transform_paint(paint: &mut Value, sat: f64, bri: f64) {
    if let Value::Object(map) = paint {
        if let Some(c) = map.get_mut("color") {
            adjust_color_val(c, sat, bri);
        }
        if let Some(stops) = map.get_mut("stops").and_then(|s| s.as_array_mut()) {
            for stop in stops {
                if let Some(c) = stop.get_mut("color") {
                    adjust_color_val(c, sat, bri);
                }
            }
        }
    }
}

pub fn transform_object(obj: &mut Value, sat: f64, bri: f64) {
    if let Value::Object(map) = obj {
        // 1. VectorCraft native appearance.items
        if let Some(app) = map.get_mut("appearance").and_then(|a| a.as_object_mut()) {
            if let Some(items) = app.get_mut("items").and_then(|i| i.as_array_mut()) {
                for item in items {
                    if let Some(paint) = item.get_mut("paint") {
                        transform_paint(paint, sat, bri);
                    }
                    if let Some(color) = item.get_mut("color") {
                        adjust_color_val(color, sat, bri);
                    }
                }
            }
        }

        // 2. Fills and strokes
        if let Some(fills) = map.get_mut("fills").and_then(|f| f.as_array_mut()) {
            for fill in fills {
                transform_paint(fill, sat, bri);
            }
        }
        if let Some(fill) = map.get_mut("fill") {
            transform_paint(fill, sat, bri);
        }
        if let Some(strokes) = map.get_mut("strokes").and_then(|s| s.as_array_mut()) {
            for stroke in strokes {
                if let Some(paint) = stroke.get_mut("paint") {
                    transform_paint(paint, sat, bri);
                } else {
                    transform_paint(stroke, sat, bri);
                }
            }
        }
        if let Some(stroke) = map.get_mut("stroke") {
            if let Some(paint) = stroke.get_mut("paint") {
                transform_paint(paint, sat, bri);
            } else {
                transform_paint(stroke, sat, bri);
            }
        }

        // 3. Recursive children
        if let Some(children) = map.get_mut("children").and_then(|c| c.as_array_mut()) {
            for child in children {
                transform_object(child, sat, bri);
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
    let sat = params_val.get("saturation").and_then(|v| v.as_f64()).unwrap_or(100.0);
    let bri = params_val.get("brightness").and_then(|v| v.as_f64()).unwrap_or(0.0);

    if let Some(objects) = doc.get_mut("objects").and_then(|o| o.as_array_mut()) {
        for obj in objects {
            transform_object(obj, sat, bri);
        }
    }

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
    fn test_appearance_recolor() {
        let mut doc = json!({
            "objects": [
                {
                    "id": 1,
                    "appearance": {
                        "items": [
                            {
                                "kind": "fill",
                                "paint": {
                                    "type": "solid",
                                    "color": {
                                        "model": "rgb",
                                        "r": 1.0,
                                        "g": 0.0,
                                        "b": 0.0
                                    }
                                }
                            }
                        ]
                    }
                }
            ]
        });

        // Desaturate to 0%
        let obj = &mut doc["objects"][0];
        transform_object(obj, 0.0, 0.0);

        let c = &obj["appearance"]["items"][0]["paint"]["color"];
        let r = c["r"].as_f64().unwrap();
        let g = c["g"].as_f64().unwrap();
        let b = c["b"].as_f64().unwrap();

        // Grayscale of pure red is ~0.2126 across all channels
        assert!((r - 0.2126).abs() < 1e-3);
        assert!((g - 0.2126).abs() < 1e-3);
        assert!((b - 0.2126).abs() < 1e-3);
    }
}
