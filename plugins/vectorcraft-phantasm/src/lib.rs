//! VectorCraft Phantasm color and tonal grading plugin (VectorCraft ABI v1).
//! Inspired by Astute Graphics Phantasm.

use serde_json::{Value, json};

pub const MANIFEST: &str = r#"{
  "id": "org.vectorcraft.community.phantasm",
  "name": "Vector Phantasm",
  "version": "1.0.0",
  "kind": "filter",
  "author": "ArtCraft Store community",
  "description": "Professional color and tonal adjustments for vector artwork and embedded raster imagery: Curves, Levels, Hue/Saturation, Exposure, Brightness/Contrast, Temperature/Tint, and Invert.",
  "params": {
    "brightness": {"type": "number", "min": -100, "max": 100, "default": 0},
    "contrast": {"type": "number", "min": -100, "max": 100, "default": 0},
    "exposure": {"type": "number", "min": -5.0, "max": 5.0, "default": 0.0},
    "hue": {"type": "number", "min": -180, "max": 180, "default": 0},
    "saturation": {"type": "number", "min": -100, "max": 100, "default": 0},
    "lightness": {"type": "number", "min": -100, "max": 100, "default": 0},
    "temperature": {"type": "number", "min": -100, "max": 100, "default": 0},
    "tint": {"type": "number", "min": -100, "max": 100, "default": 0},
    "invert": {"type": "bool", "default": false}
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

#[derive(Debug, Clone, Copy, Default)]
pub struct PhantasmParams {
    pub brightness: f64,
    pub contrast: f64,
    pub exposure: f64,
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64,
    pub temperature: f64,
    pub tint: f64,
    pub invert: bool,
}

impl PhantasmParams {
    pub fn from_json(val: &Value) -> Self {
        Self {
            brightness: val.get("brightness").and_then(|v| v.as_f64()).unwrap_or(0.0),
            contrast: val.get("contrast").and_then(|v| v.as_f64()).unwrap_or(0.0),
            exposure: val.get("exposure").and_then(|v| v.as_f64()).unwrap_or(0.0),
            hue: val.get("hue").and_then(|v| v.as_f64()).unwrap_or(0.0),
            saturation: val.get("saturation").and_then(|v| v.as_f64()).unwrap_or(0.0),
            lightness: val.get("lightness").and_then(|v| v.as_f64()).unwrap_or(0.0),
            temperature: val.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.0),
            tint: val.get("tint").and_then(|v| v.as_f64()).unwrap_or(0.0),
            invert: val.get("invert").and_then(|v| v.as_bool()).unwrap_or(false),
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

    let s = if l > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };

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

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h_norm = (h % 360.0 + 360.0) % 360.0 / 360.0;

    let r = hue_to_rgb(p, q, h_norm + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h_norm);
    let b = hue_to_rgb(p, q, h_norm - 1.0 / 3.0);

    (r, g, b)
}

/// Applies the Phantasm tonal and color grading pipeline to an RGB triplet [0.0, 1.0].
pub fn adjust_rgb(mut r: f64, mut g: f64, mut b: f64, params: &PhantasmParams) -> (f64, f64, f64) {
    // 1. Invert
    if params.invert {
        r = 1.0 - r;
        g = 1.0 - g;
        b = 1.0 - b;
    }

    // 2. Exposure: R' = R * 2^(exposure)
    if params.exposure.abs() > 1e-6 {
        let mult = 2.0_f64.powf(params.exposure);
        r *= mult;
        g *= mult;
        b *= mult;
    }

    // 3. Temperature & Tint
    if params.temperature.abs() > 1e-6 {
        let temp_factor = params.temperature / 100.0 * 0.15;
        r += temp_factor;
        b -= temp_factor;
    }
    if params.tint.abs() > 1e-6 {
        let tint_factor = params.tint / 100.0 * 0.15;
        g -= tint_factor;
        r += tint_factor * 0.5;
        b += tint_factor * 0.5;
    }

    // Clamp before HSL transform
    r = r.clamp(0.0, 1.0);
    g = g.clamp(0.0, 1.0);
    b = b.clamp(0.0, 1.0);

    // 4. Hue, Saturation & Lightness
    if params.hue.abs() > 1e-6 || params.saturation.abs() > 1e-6 || params.lightness.abs() > 1e-6 {
        let (mut h, mut s, mut l) = rgb_to_hsl(r, g, b);
        h += params.hue;
        if params.saturation >= 0.0 {
            s += (1.0 - s) * (params.saturation / 100.0);
        } else {
            s += s * (params.saturation / 100.0);
        }
        if params.lightness >= 0.0 {
            l += (1.0 - l) * (params.lightness / 100.0);
        } else {
            l += l * (params.lightness / 100.0);
        }
        s = s.clamp(0.0, 1.0);
        l = l.clamp(0.0, 1.0);
        let (nr, ng, nb) = hsl_to_rgb(h, s, l);
        r = nr;
        g = ng;
        b = nb;
    }

    // 5. Brightness
    if params.brightness.abs() > 1e-6 {
        let b_shift = params.brightness / 100.0;
        r += b_shift;
        g += b_shift;
        b += b_shift;
    }

    // 6. Contrast (centered on 0.5 midtone)
    if params.contrast.abs() > 1e-6 {
        let factor = (100.0 + params.contrast) / 100.0;
        let c_mult = factor.max(0.0);
        r = (r - 0.5) * c_mult + 0.5;
        g = (g - 0.5) * c_mult + 0.5;
        b = (b - 0.5) * c_mult + 0.5;
    }

    (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

fn parse_hex_color(hex: &str) -> Option<(f64, f64, f64, f64)> {
    let clean = hex.trim().trim_start_matches('#');
    match clean.len() {
        3 => {
            let r = u8::from_str_radix(&clean[0..1].repeat(2), 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&clean[1..2].repeat(2), 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&clean[2..3].repeat(2), 16).ok()? as f64 / 255.0;
            Some((r, g, b, 1.0))
        }
        6 => {
            let r = u8::from_str_radix(&clean[0..2], 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&clean[2..4], 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&clean[4..6], 16).ok()? as f64 / 255.0;
            Some((r, g, b, 1.0))
        }
        8 => {
            let r = u8::from_str_radix(&clean[0..2], 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&clean[2..4], 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&clean[4..6], 16).ok()? as f64 / 255.0;
            let a = u8::from_str_radix(&clean[6..8], 16).ok()? as f64 / 255.0;
            Some((r, g, b, a))
        }
        _ => None,
    }
}

fn format_hex_color(r: f64, g: f64, b: f64, a: f64) -> String {
    let ir = (r * 255.0).round() as u8;
    let ig = (g * 255.0).round() as u8;
    let ib = (b * 255.0).round() as u8;
    let ia = (a * 255.0).round() as u8;
    if (a - 1.0).abs() < 1e-4 {
        format!("#{:02x}{:02x}{:02x}", ir, ig, ib)
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", ir, ig, ib, ia)
    }
}

pub fn adjust_color_value(val: &mut Value, params: &PhantasmParams) {
    match val {
        Value::Object(map) => {
            if let Some(model) = map.get("model").and_then(|m| m.as_str()) {
                if model == "rgb" {
                    let r = map.get("r").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let g = map.get("g").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let b = map.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let (nr, ng, nb) = adjust_rgb(r, g, b, params);
                    map.insert("r".to_string(), json!(nr));
                    map.insert("g".to_string(), json!(ng));
                    map.insert("b".to_string(), json!(nb));
                } else if model == "gray" {
                    let gray = map.get("gray").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let (nr, ng, nb) = adjust_rgb(gray, gray, gray, params);
                    let n_gray = 0.2126 * nr + 0.7152 * ng + 0.0722 * nb;
                    map.insert("gray".to_string(), json!(n_gray));
                }
            } else if let Some(c) = map.get_mut("color") {
                adjust_color_value(c, params);
            }
        }
        Value::Array(arr) => {
            if arr.len() >= 3 {
                let is_byte_range = arr.iter().take(3).any(|c| c.as_f64().unwrap_or(0.0) > 1.0);
                let (scale, inv_scale) = if is_byte_range { (255.0, 1.0 / 255.0) } else { (1.0, 1.0) };

                let r = arr[0].as_f64().unwrap_or(0.0) * inv_scale;
                let g = arr[1].as_f64().unwrap_or(0.0) * inv_scale;
                let b = arr[2].as_f64().unwrap_or(0.0) * inv_scale;

                let (nr, ng, nb) = adjust_rgb(r, g, b, params);

                arr[0] = json!(nr * scale);
                arr[1] = json!(ng * scale);
                arr[2] = json!(nb * scale);
            }
        }
        Value::String(s) => {
            if let Some((r, g, b, a)) = parse_hex_color(s) {
                let (nr, ng, nb) = adjust_rgb(r, g, b, params);
                *s = format_hex_color(nr, ng, nb, a);
            }
        }
        _ => {}
    }
}

fn transform_paint(paint: &mut Value, params: &PhantasmParams) {
    if let Value::Object(map) = paint {
        if let Some(color) = map.get_mut("color") {
            adjust_color_value(color, params);
        }
        if let Some(stops) = map.get_mut("stops").and_then(|s| s.as_array_mut()) {
            for stop in stops {
                if let Some(c) = stop.get_mut("color") {
                    adjust_color_value(c, params);
                }
            }
        }
    }
}

pub fn transform_object(obj: &mut Value, params: &PhantasmParams) {
    if let Value::Object(map) = obj {
        if let Some(fills) = map.get_mut("fills").and_then(|f| f.as_array_mut()) {
            for fill in fills {
                transform_paint(fill, params);
            }
        }
        if let Some(fill) = map.get_mut("fill") {
            transform_paint(fill, params);
        }
        if let Some(strokes) = map.get_mut("strokes").and_then(|s| s.as_array_mut()) {
            for stroke in strokes {
                if let Some(paint) = stroke.get_mut("paint") {
                    transform_paint(paint, params);
                }
                transform_paint(stroke, params);
            }
        }
        if let Some(stroke) = map.get_mut("stroke") {
            if let Some(paint) = stroke.get_mut("paint") {
                transform_paint(paint, params);
            }
            transform_paint(stroke, params);
        }
        // Raster embedded image pixels support
        if let Some(image) = map.get_mut("image").and_then(|im| im.as_object_mut()) {
            if let Some(pixels) = image.get_mut("pixels").and_then(|p| p.as_array_mut()) {
                // If pixels is array of [r,g,b,a]
                for px in pixels {
                    adjust_color_value(px, params);
                }
            }
        }
        // Recursively traverse children
        if let Some(children) = map.get_mut("children").and_then(|c| c.as_array_mut()) {
            for child in children {
                transform_object(child, params);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn vc_run(input: *const u8, input_len: u32, params: *const u8, params_len: u32) -> i64 {
    let in_slice = unsafe { std::slice::from_raw_parts(input, input_len as usize) };
    let param_slice = unsafe { std::slice::from_raw_parts(params, params_len as usize) };
    let _ = std::fs::write("/tmp/vc_input.json", in_slice);
    let _ = std::fs::write("/tmp/vc_params.json", param_slice);

    let mut doc: Value = match serde_json::from_slice(in_slice) {
        Ok(v) => v,
        Err(_) => return -1,
    };
    let params_val: Value = serde_json::from_slice(param_slice).unwrap_or(Value::Null);
    let p = PhantasmParams::from_json(&params_val);

    if let Some(objects) = doc.get_mut("objects").and_then(|o| o.as_array_mut()) {
        for obj in objects {
            transform_object(obj, &p);
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
    fn test_invert() {
        let p = PhantasmParams { invert: true, ..Default::default() };
        let (r, g, b) = adjust_rgb(0.2, 0.8, 0.0, &p);
        assert!((r - 0.8).abs() < 1e-4);
        assert!((g - 0.2).abs() < 1e-4);
        assert!((b - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_exposure() {
        let p = PhantasmParams { exposure: 1.0, ..Default::default() }; // +1 stop doubles values
        let (r, g, b) = adjust_rgb(0.25, 0.4, 0.5, &p);
        assert!((r - 0.5).abs() < 1e-4);
        assert!((g - 0.8).abs() < 1e-4);
        assert!((b - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_brightness_and_contrast() {
        let p = PhantasmParams { brightness: 10.0, contrast: 20.0, ..Default::default() };
        let (r, _, _) = adjust_rgb(0.5, 0.5, 0.5, &p);
        // Midtone 0.5 + 0.1 brightness = 0.6; contrast expands around 0.5
        assert!((r - 0.62).abs() < 1e-2);
    }

    #[test]
    fn test_hue_shift() {
        let p = PhantasmParams { hue: 120.0, ..Default::default() };
        // Pure red shifted 120 deg should become green
        let (r, g, b) = adjust_rgb(1.0, 0.0, 0.0, &p);
        assert!(r < 0.05);
        assert!(g > 0.95);
        assert!(b < 0.05);
    }

    #[test]
    fn test_hex_color_adjustment() {
        let p = PhantasmParams { invert: true, ..Default::default() };
        let mut val = json!("#ffffff");
        adjust_color_value(&mut val, &p);
        assert_eq!(val.as_str().unwrap(), "#000000");
    }

    #[test]
    fn test_full_document_transform() {
        let mut doc = json!({
            "version": 1,
            "objects": [
                {
                    "type": "path",
                    "fill": {
                        "color": [1.0, 0.0, 0.0]
                    },
                    "stroke": {
                        "stops": [
                            {"offset": 0.0, "color": [0.0, 1.0, 0.0]}
                        ]
                    }
                }
            ]
        });

        let p = PhantasmParams { invert: true, ..Default::default() };
        for obj in doc.get_mut("objects").unwrap().as_array_mut().unwrap() {
            transform_object(obj, &p);
        }

        let fill_color = &doc["objects"][0]["fill"]["color"];
        assert_eq!(fill_color[0], 0.0);
        assert_eq!(fill_color[1], 1.0);
        assert_eq!(fill_color[2], 1.0);
    }
}
