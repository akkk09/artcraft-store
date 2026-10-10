//! VectorCraft Recolor & Tone object filter plugin (VectorCraft ABI v1).

use serde_json::{Value, json};

const MANIFEST: &str = r#"{
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

fn adjust_color_val(color: &mut Value, sat: f64, bri: f64) {
    if let Value::Array(arr) = color {
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
}

fn transform_object(obj: &mut Value, sat: f64, bri: f64) {
    if let Value::Object(map) = obj {
        if let Some(fill) = map.get_mut("fill") {
            if let Some(c) = fill.get_mut("color") {
                adjust_color_val(c, sat, bri);
            }
        }
        if let Some(stroke) = map.get_mut("stroke") {
            if let Some(c) = stroke.get_mut("color") {
                adjust_color_val(c, sat, bri);
            }
        }
        if let Some(children) = map.get_mut("children") {
            if let Value::Array(list) = children {
                for child in list {
                    transform_object(child, sat, bri);
                }
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
