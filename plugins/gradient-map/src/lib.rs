#![no_std]

#[cfg(test)]
extern crate std;

use core::slice;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}

static MANIFEST: &[u8] = br#"{
  "id":"org.photocraft.community.gradient-map",
  "name":"Gradient Map & Duotone",
  "version":"0.1.0",
  "kind":"filter",
  "author":"ArtCraft Store community",
  "description":"Maps image luminance to a selectable two-colour gradient while preserving alpha.",
  "params":{
    "preset":{"type":"choice","options":["black-white","warm-cream","blue-orange","purple-pink","teal-yellow"],"default":"blue-orange"},
    "intensity":{"type":"int","min":0,"max":100,"default":100}
  },
  "overlap":0,
  "area":"content"
}"#;

static mut HEAP_NEXT: usize = 65536;

#[no_mangle]
pub extern "C" fn pc_abi_version() -> i32 { 1 }

#[no_mangle]
pub extern "C" fn pc_manifest() -> i64 {
    ((MANIFEST.len() as i64) << 32) | (MANIFEST.as_ptr() as u32 as i64)
}

#[no_mangle]
pub extern "C" fn pc_alloc(size: i32) -> i32 {
    if size <= 0 { return 0; }
    let size = size as usize;
    let start = unsafe { (HEAP_NEXT + 7) & !7usize };
    let end = match start.checked_add(size) { Some(value) => value, None => return 0 };

    #[cfg(target_arch = "wasm32")]
    {
        let pages_needed = end.saturating_add(65535) / 65536;
        let pages_now = core::arch::wasm32::memory_size(0);
        if pages_needed > pages_now {
            let grown = core::arch::wasm32::memory_grow(0, pages_needed - pages_now);
            if grown == usize::MAX { return 0; }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = end;
        return 0;
    }

    unsafe { HEAP_NEXT = end; }
    start as i32
}

#[derive(Clone, Copy)]
struct Color { r: f32, g: f32, b: f32 }

const PRESETS: [([u8; 3], [u8; 3]); 5] = [
    ([0, 0, 0], [255, 255, 255]),
    ([20, 15, 18], [255, 236, 196]),
    ([10, 24, 96], [255, 126, 32]),
    ([38, 12, 75], [255, 110, 170]),
    ([0, 50, 48], [240, 230, 120]),
];

fn preset_index(params: &[u8]) -> usize {
    let mut i = 0;
    while i + 8 < params.len() {
        if params[i..].starts_with(b"\"preset\"") {
            let rest = &params[i + 8..];
            if let Some(colon) = rest.iter().position(|&byte| byte == b':') {
                let value = &rest[colon + 1..];
                if value.windows(11).any(|w| w == b"black-white") { return 0; }
                if value.windows(10).any(|w| w == b"warm-cream") { return 1; }
                if value.windows(11).any(|w| w == b"blue-orange") { return 2; }
                if value.windows(11).any(|w| w == b"purple-pink") { return 3; }
                if value.windows(10).any(|w| w == b"teal-yellow") { return 4; }
            }
            return 2;
        }
        i += 1;
    }
    2
}

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
            let mut number = 0i32;
            let mut found = false;
            while j < params.len() && params[j].is_ascii_digit() {
                number = number.saturating_mul(10).saturating_add((params[j] - b'0') as i32);
                found = true;
                j += 1;
            }
            return if found { number } else { default };
        }
        i += 1;
    }
    default
}

fn rgb_from_u8(value: [u8; 3]) -> Color {
    Color { r: value[0] as f32 / 255.0, g: value[1] as f32 / 255.0, b: value[2] as f32 / 255.0 }
}

fn luminance(color: Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}

fn interpolate(low: Color, high: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: low.r + (high.r - low.r) * t,
        g: low.g + (high.g - low.g) * t,
        b: low.b + (high.b - low.b) * t,
    }
}

#[no_mangle]
pub unsafe extern "C" fn pc_filter(
    buf: i32, buf_len: i32, width: i32, height: i32, channels: i32,
    format: i32, params_ptr: i32, params_len: i32,
) -> i32 {
    if buf <= 0 || width <= 0 || height <= 0 || channels <= 0 || params_ptr <= 0 || params_len < 0 {
        return 1;
    }
    let count = match (width as usize).checked_mul(height as usize)
        .and_then(|value| value.checked_mul(channels as usize)) {
        Some(value) => value,
        None => return 2,
    };
    if count.checked_mul(4) != Some(buf_len as usize) { return 3; }

    // ABI v1 uses: 1 grayscale, 2 indexed stored as RGB, 3 RGB, 8 duotone stored as gray.
    let mode = (format >> 16) & 0xff;
    if !matches!(mode, 1 | 2 | 3 | 8) { return 5; }

    let params = slice::from_raw_parts(params_ptr as *const u8, params_len as usize);
    let intensity = int_param(params, b"intensity", 100).clamp(0, 100) as f32 / 100.0;
    if intensity == 0.0 { return 0; }

    let (low_bytes, high_bytes) = PRESETS[preset_index(params)];
    let low = rgb_from_u8(low_bytes);
    let high = rgb_from_u8(high_bytes);
    let ch = channels as usize;
    let has_alpha = (format & 0x100) != 0 && ch > 1;
    let color_ch = if has_alpha { ch - 1 } else { ch };
    if matches!(mode, 2 | 3) && color_ch != 3 { return 6; }
    if matches!(mode, 1 | 8) && color_ch != 1 { return 6; }

    let pixels = slice::from_raw_parts_mut(buf as *mut f32, count);
    for pixel in pixels.chunks_exact_mut(ch) {
        if has_alpha && pixel[ch - 1] <= 0.0 { continue; }
        let original = if color_ch == 3 {
            Color { r: pixel[0], g: pixel[1], b: pixel[2] }
        } else {
            Color { r: pixel[0], g: pixel[0], b: pixel[0] }
        };
        let mapped = interpolate(low, high, luminance(original));
        if color_ch == 3 {
            pixel[0] = original.r + (mapped.r - original.r) * intensity;
            pixel[1] = original.g + (mapped.g - original.g) * intensity;
            pixel[2] = original.b + (mapped.b - original.b) * intensity;
        } else {
            let mapped_gray = luminance(mapped);
            pixel[0] = original.r + (mapped_gray - original.r) * intensity;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_endpoints_are_stable() {
        let low = rgb_from_u8([0, 0, 0]);
        let high = rgb_from_u8([255, 255, 255]);
        assert_eq!(interpolate(low, high, 0.0).r, 0.0);
        assert_eq!(interpolate(low, high, 1.0).r, 1.0);
    }

    #[test]
    fn interpolation_clamps_input() {
        let low = rgb_from_u8([0, 0, 0]);
        let high = rgb_from_u8([255, 255, 255]);
        assert_eq!(interpolate(low, high, -1.0).r, 0.0);
        assert_eq!(interpolate(low, high, 2.0).r, 1.0);
    }

    #[test]
    fn luminance_tracks_black_and_white() {
        assert_eq!(luminance(rgb_from_u8([0, 0, 0])), 0.0);
        assert_eq!(luminance(rgb_from_u8([255, 255, 255])), 1.0);
    }

    #[test]
    fn intensity_parser_clamps_out_of_range_values() {
        assert_eq!(int_param(br#"{"intensity":140}"#, b"intensity", 100).clamp(0, 100), 100);
        assert_eq!(int_param(br#"{"intensity":5}"#, b"intensity", 100).clamp(0, 100), 5);
    }

    #[test]
    fn preset_parser_selects_expected_gradient() {
        assert_eq!(preset_index(br#"{"preset":"warm-cream"}"#), 1);
        assert_eq!(preset_index(br#"{"preset":"teal-yellow"}"#), 4);
        assert_eq!(preset_index(br#"{}"#), 2);
    }
}
