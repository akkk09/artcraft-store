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
  "id":"org.photocraft.community.white-balance",
  "name":"White Balance",
  "version":"0.1.0",
  "kind":"filter",
  "author":"ArtCraft Store community",
  "description":"Corrects warm/cool color casts with temperature and tint controls.",
  "params":{
    "temperature":{"type":"int","min":-100,"max":100,"default":0},
    "tint":{"type":"int","min":-100,"max":100,"default":0}
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
            if negative || params.get(j) == Some(&b'+') { j += 1; }
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

fn gains(temperature: f32, tint: f32) -> (f32, f32, f32) {
    let red = 1.0 + temperature * 0.25 + tint * 0.12;
    let green = 1.0 - tint * 0.18;
    let blue = 1.0 - temperature * 0.25 + tint * 0.12;
    (
        if red < 0.1 { 0.1 } else { red },
        if green < 0.1 { 0.1 } else { green },
        if blue < 0.1 { 0.1 } else { blue },
    )
}

fn apply_white_balance(pixel: &mut [f32], color_channels: usize, has_alpha: bool,
                       temperature: f32, tint: f32) {
    if has_alpha && pixel[color_channels] <= 0.0 { return; }
    if color_channels != 3 || (temperature == 0.0 && tint == 0.0) { return; }
    let (r, g, b) = gains(temperature, tint);
    pixel[0] *= r;
    pixel[1] *= g;
    pixel[2] *= b;
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
    let temperature = int_param(params, b"temperature", 0).clamp(-100, 100) as f32 / 100.0;
    let tint = int_param(params, b"tint", 0).clamp(-100, 100) as f32 / 100.0;
    if temperature == 0.0 && tint == 0.0 { return 0; }

    let pixels = slice::from_raw_parts_mut(buf as *mut f32, count);
    for pixel in pixels.chunks_exact_mut(ch) {
        apply_white_balance(pixel, color_channels, has_alpha, temperature, tint);
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.0001, "actual={actual}, expected={expected}");
    }

    #[test]
    fn neutral_settings_are_identity() {
        let mut pixel = [0.2, 0.4, 0.8];
        let before = pixel;
        apply_white_balance(&mut pixel, 3, false, 0.0, 0.0);
        assert_eq!(pixel, before);
    }

    #[test]
    fn warm_temperature_boosts_red_and_reduces_blue() {
        let (r, _, b) = gains(1.0, 0.0);
        assert!(r > 1.0);
        assert!(b < 1.0);
    }

    #[test]
    fn cool_temperature_reduces_red_and_boosts_blue() {
        let (r, _, b) = gains(-1.0, 0.0);
        assert!(r < 1.0);
        assert!(b > 1.0);
    }

    #[test]
    fn positive_tint_moves_toward_magenta() {
        let (r, g, b) = gains(0.0, 1.0);
        assert!(r > 1.0 && b > 1.0 && g < 1.0);
    }

    #[test]
    fn alpha_is_preserved() {
        let mut pixel = [0.2, 0.4, 0.6, 0.37];
        apply_white_balance(&mut pixel, 3, true, 0.5, -0.2);
        close(pixel[3], 0.37);
    }

    #[test]
    fn fully_transparent_pixel_is_unchanged() {
        let mut pixel = [0.2, 0.4, 0.6, 0.0];
        let before = pixel;
        apply_white_balance(&mut pixel, 3, true, 0.5, 0.2);
        assert_eq!(pixel, before);
    }

    #[test]
    fn grayscale_is_unchanged() {
        let mut pixel = [0.4];
        apply_white_balance(&mut pixel, 1, false, 0.8, 0.8);
        close(pixel[0], 0.4);
    }

    #[test]
    fn extreme_values_keep_gains_positive() {
        let (r, g, b) = gains(-1.0, -1.0);
        assert!(r >= 0.1 && g >= 0.1 && b >= 0.1);
    }

    #[test]
    fn parameter_parser_handles_signed_values() {
        assert_eq!(int_param(br#"{"temperature":-75}"#, b"temperature", 0), -75);
        assert_eq!(int_param(br#"{"tint":40}"#, b"tint", 0), 40);
    }
}
