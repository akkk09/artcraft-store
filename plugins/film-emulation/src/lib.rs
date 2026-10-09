#![no_std]

#[cfg(test)]
extern crate std;

use core::{ptr, slice};

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}

static MANIFEST: &[u8] = br#"{
  "id":"org.photocraft.community.film-emulation",
  "name":"Film Emulation Toolkit",
  "version":"0.1.0",
  "kind":"filter",
  "author":"ArtCraft Store community",
  "description":"Applies compact film-inspired color looks with deterministic grain.",
  "params":{
    "preset":{"type":"choice","options":["classic","warm","cool","faded","cinematic"],"default":"classic"},
    "intensity":{"type":"int","min":0,"max":100,"default":75},
    "grain":{"type":"int","min":0,"max":100,"default":8}
  },
  "overlap":0,
  "area":"content"
}"#;

#[cfg(target_arch = "wasm32")]
static mut HEAP_NEXT: usize = 65536;

#[no_mangle]
pub extern "C" fn pc_abi_version() -> i32 { 1 }

#[no_mangle]
pub extern "C" fn pc_manifest() -> i64 {
    ((MANIFEST.len() as i64) << 32) | (MANIFEST.as_ptr() as u32 as i64)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn pc_alloc(size: i32) -> i32 {
    if size <= 0 { return 0; }
    let size = size as usize;
    let start = unsafe { (HEAP_NEXT + 7) & !7usize };
    let end = match start.checked_add(size) { Some(value) => value, None => return 0 };
    let pages_needed = end.saturating_add(65535) / 65536;
    let pages_now = core::arch::wasm32::memory_size(0);
    if pages_needed > pages_now {
        if core::arch::wasm32::memory_grow(0, pages_needed - pages_now) == usize::MAX { return 0; }
    }
    unsafe { HEAP_NEXT = end; }
    start as i32
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub extern "C" fn pc_alloc(_size: i32) -> i32 { 0 }

fn int_param(params: &[u8], key: &[u8], default: i32) -> i32 {
    let mut i = 0;
    while i + key.len() + 2 < params.len() {
        if params[i] == b'"'
            && params.get(i + 1..i + 1 + key.len()) == Some(key)
            && params.get(i + key.len() + 1) == Some(&b'"')
        {
            let mut j = i + key.len() + 2;
            while j < params.len() && params[j].is_ascii_whitespace() { j += 1; }
            if params.get(j) != Some(&b':') { return default; }
            j += 1;
            while j < params.len() && params[j].is_ascii_whitespace() { j += 1; }
            let negative = params.get(j) == Some(&b'-');
            if negative { j += 1; }
            let mut number = 0i32;
            let mut found = false;
            while j < params.len() && params[j].is_ascii_digit() {
                number = number.saturating_mul(10).saturating_add((params[j] - b'0') as i32);
                found = true;
                j += 1;
            }
            if !found { return default; }
            return if negative { number.saturating_neg() } else { number };
        }
        i += 1;
    }
    default
}

fn preset_index(params: &[u8]) -> usize {
    let mut i = 0;
    while i + 8 < params.len() {
        if params[i..].starts_with(b"\"preset\"") {
            let rest = &params[i + 8..];
            if let Some(colon) = rest.iter().position(|&byte| byte == b':') {
                let value = &rest[colon + 1..];
                if value.windows(7).any(|w| w == b"classic") { return 0; }
                if value.windows(4).any(|w| w == b"warm") { return 1; }
                if value.windows(3).any(|w| w == b"cool") { return 2; }
                if value.windows(5).any(|w| w == b"faded") { return 3; }
                if value.windows(9).any(|w| w == b"cinematic") { return 4; }
            }
            return 0;
        }
        i += 1;
    }
    0
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Color { r: f32, g: f32, b: f32 }

fn luminance(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

fn grade(mut c: Color, preset: usize) -> Color {
    let l = luminance(c);
    match preset {
        1 => {
            c.r += 0.08 * (1.0 - l);
            c.g += 0.025 * (1.0 - l);
            c.b -= 0.07 * (0.3 + l);
        }
        2 => {
            c.r -= 0.06 * (0.2 + l);
            c.g += 0.01;
            c.b += 0.07 * (1.0 - l);
        }
        3 => {
            c.r = c.r * 0.82 + 0.08;
            c.g = c.g * 0.82 + 0.08;
            c.b = c.b * 0.82 + 0.08;
        }
        4 => {
            c.r = (c.r - 0.5) * 1.12 + 0.5 + 0.04 * l;
            c.g = (c.g - 0.5) * 1.12 + 0.5 + 0.02 * (1.0 - l);
            c.b = (c.b - 0.5) * 1.12 + 0.5 + 0.05 * (1.0 - l) - 0.03 * l;
        }
        _ => {
            c.r = (c.r - 0.5) * 1.08 + 0.5 + 0.025 * (1.0 - l);
            c.g = (c.g - 0.5) * 1.08 + 0.5 + 0.005;
            c.b = (c.b - 0.5) * 1.08 + 0.5 - 0.02 * l;
        }
    }
    c
}

fn noise(x: i32, y: i32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x9E3779B9)
        ^ (y as u32).wrapping_mul(0x85EBCA6B)
        ^ 0xC2B2AE35;
    n ^= n >> 16;
    n = n.wrapping_mul(0x7FEB352D);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846CA68B);
    n ^= n >> 16;
    (n as f32 / u32::MAX as f32) - 0.5
}

fn process_pixels(
    pixels: &mut [f32],
    width: usize,
    height: usize,
    channels: usize,
    color_channels: usize,
    has_alpha: bool,
    preset: usize,
    intensity: f32,
    grain: f32,
    origin_x: i32,
    origin_y: i32,
) {
    let alpha_index = if has_alpha { Some(channels - 1) } else { None };
    for y in 0..height {
        for x in 0..width {
            let p = (y * width + x) * channels;
            if let Some(a) = alpha_index {
                if pixels[p + a] <= 0.0 { continue; }
            }
            let original = if color_channels == 3 {
                Color { r: pixels[p], g: pixels[p + 1], b: pixels[p + 2] }
            } else {
                Color { r: pixels[p], g: pixels[p], b: pixels[p] }
            };
            let mapped = grade(original, preset);
            let g = noise(origin_x.wrapping_add(x as i32), origin_y.wrapping_add(y as i32)) * grain * 0.08;
            let r = original.r + (mapped.r - original.r) * intensity + g;
            let green = original.g + (mapped.g - original.g) * intensity + g;
            let b = original.b + (mapped.b - original.b) * intensity + g;
            if color_channels == 3 {
                pixels[p] = r;
                pixels[p + 1] = green;
                pixels[p + 2] = b;
            } else {
                pixels[p] = original.r + (luminance(mapped) - original.r) * intensity + g;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pc_filter(
    buf: i32, buf_len: i32, width: i32, height: i32, channels: i32,
    format: i32, params_ptr: i32, params_len: i32,
) -> i32 {
    if buf <= 0 || buf_len < 0 || width <= 0 || height <= 0 || channels <= 0
        || params_ptr <= 0 || params_len < 0 {
        return 1;
    }
    let count = match (width as usize).checked_mul(height as usize)
        .and_then(|value| value.checked_mul(channels as usize)) {
        Some(value) => value,
        None => return 2,
    };
    if count.checked_mul(4) != Some(buf_len as usize) { return 3; }

    let mode = (format >> 16) & 0xff;
    if !matches!(mode, 1 | 2 | 3 | 8) { return 5; }
    let ch = channels as usize;
    let has_alpha = (format & 0x100) != 0 && ch > 1;
    let color_channels = ch - usize::from(has_alpha);
    if matches!(mode, 2 | 3) && color_channels != 3 { return 6; }
    if matches!(mode, 1 | 8) && color_channels != 1 { return 6; }

    let params = slice::from_raw_parts(params_ptr as *const u8, params_len as usize);
    let preset = preset_index(params);
    let intensity = int_param(params, b"intensity", 75).clamp(0, 100) as f32 / 100.0;
    let grain = int_param(params, b"grain", 8).clamp(0, 100) as f32 / 100.0;
    let origin_x = int_param(params, b"x", 0);
    let origin_y = int_param(params, b"y", 0);
    if intensity == 0.0 && grain == 0.0 { return 0; }

    let pixels = slice::from_raw_parts_mut(buf as *mut f32, count);
    process_pixels(pixels, width as usize, height as usize, ch, color_channels, has_alpha,
        preset, intensity, grain, origin_x, origin_y);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.0001, "actual={actual}, expected={expected}");
    }

    #[test]
    fn zero_intensity_and_grain_are_identity() {
        let mut pixels = [0.2, 0.4, 0.6];
        let before = pixels;
        process_pixels(&mut pixels, 1, 1, 3, 3, false, 0, 0.0, 0.0, 0, 0);
        assert_eq!(pixels, before);
    }

    #[test]
    fn faded_preset_lifts_shadows_and_compresses_highlights() {
        let mut pixels = [0.0, 0.5, 1.0];
        process_pixels(&mut pixels, 1, 1, 3, 3, false, 3, 1.0, 0.0, 0, 0);
        close(pixels[0], 0.08);
        close(pixels[1], 0.49);
        close(pixels[2], 0.9);
    }

    #[test]
    fn grain_is_deterministic_for_same_coordinates() {
        let mut a = [0.5, 0.5, 0.5];
        let mut b = [0.5, 0.5, 0.5];
        process_pixels(&mut a, 1, 1, 3, 3, false, 0, 0.0, 1.0, 20, 30);
        process_pixels(&mut b, 1, 1, 3, 3, false, 0, 0.0, 1.0, 20, 30);
        assert_eq!(a, b);
    }

    #[test]
    fn grain_changes_with_position() {
        assert_ne!(noise(1, 1), noise(2, 1));
    }

    #[test]
    fn alpha_is_preserved_and_transparent_pixels_are_unchanged() {
        let mut pixels = [0.2, 0.4, 0.6, 0.0, 0.2, 0.4, 0.6, 0.37];
        let before = pixels;
        process_pixels(&mut pixels, 2, 1, 4, 3, true, 1, 1.0, 1.0, 0, 0);
        assert_eq!(&pixels[0..4], &before[0..4]);
        assert_eq!(pixels[7], before[7]);
    }

    #[test]
    fn preset_parser_reads_all_supported_looks() {
        assert_eq!(preset_index(br#"{"preset":"classic"}"#), 0);
        assert_eq!(preset_index(br#"{"preset":"warm"}"#), 1);
        assert_eq!(preset_index(br#"{"preset":"cool"}"#), 2);
        assert_eq!(preset_index(br#"{"preset":"faded"}"#), 3);
        assert_eq!(preset_index(br#"{"preset":"cinematic"}"#), 4);
    }

    #[test]
    fn signed_coordinate_parameters_parse() {
        assert_eq!(int_param(br#"{"x":-10,"y":256}"#, b"x", 0), -10);
        assert_eq!(int_param(br#"{"x":-10,"y":256}"#, b"y", 0), 256);
    }
}
