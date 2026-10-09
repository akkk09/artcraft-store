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
  "id":"org.photocraft.community.vignette",
  "name":"Vignette",
  "version":"0.1.0",
  "kind":"filter",
  "author":"ArtCraft Store community",
  "description":"Adds a soft, adjustable edge darkening with a smooth radial falloff while preserving alpha.",
  "params":{
    "strength":{"type":"int","min":0,"max":100,"default":45},
    "radius":{"type":"int","min":0,"max":100,"default":70},
    "softness":{"type":"int","min":1,"max":100,"default":55}
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

fn vignette_factor(dx: f32, dy: f32, radius: f32, softness: f32, strength: f32) -> f32 {
    if strength <= 0.0 { return 1.0; }
    let distance_sq = dx * dx + dy * dy;
    let inner = radius * radius;
    if distance_sq <= inner { return 1.0; }
    let outer_radius = radius + softness;
    let outer = outer_radius * outer_radius;
    if distance_sq >= outer { return 1.0 - strength; }
    let t = ((distance_sq - inner) / (outer - inner)).clamp(0.0, 1.0);
    let smooth = t * t * (3.0 - 2.0 * t);
    1.0 - strength * smooth
}

fn apply_vignette(
    pixels: &mut [f32],
    width: usize,
    height: usize,
    channels: usize,
    color_channels: usize,
    has_alpha: bool,
    strength: f32,
    radius: f32,
    softness: f32,
) {
    if width == 0 || height == 0 || channels == 0 || strength <= 0.0 { return; }
    let min_dimension = width.min(height) as f32;
    let half_min = (min_dimension * 0.5).max(0.5);
    let cx = width as f32 * 0.5;
    let cy = height as f32 * 0.5;
    let alpha_index = if has_alpha { Some(channels - 1) } else { None };

    for y in 0..height {
        for x in 0..width {
            let p = (y * width + x) * channels;
            if let Some(a) = alpha_index {
                if pixels[p + a] <= 0.0 { continue; }
            }
            let dx = (x as f32 + 0.5 - cx) / half_min;
            let dy = (y as f32 + 0.5 - cy) / half_min;
            let factor = vignette_factor(dx, dy, radius, softness, strength);
            for c in 0..color_channels {
                pixels[p + c] *= factor;
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
    let strength = int_param(params, b"strength", 45).clamp(0, 100) as f32 / 100.0;
    let radius = 0.2 + int_param(params, b"radius", 70).clamp(0, 100) as f32 / 100.0 * 1.2;
    let softness = 0.05 + int_param(params, b"softness", 55).clamp(1, 100) as f32 / 100.0 * 0.8;
    if strength == 0.0 { return 0; }

    let pixels = slice::from_raw_parts_mut(buf as *mut f32, count);
    apply_vignette(pixels, width as usize, height as usize, ch, color_channels, has_alpha, strength, radius, softness);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.0001, "actual={actual}, expected={expected}");
    }

    #[test]
    fn zero_strength_is_identity() {
        close(vignette_factor(1.0, 1.0, 0.5, 0.5, 0.0), 1.0);
    }

    #[test]
    fn center_is_not_darkened() {
        close(vignette_factor(0.0, 0.0, 0.7, 0.4, 0.8), 1.0);
    }

    #[test]
    fn outside_falloff_is_darkened_by_strength() {
        close(vignette_factor(1.5, 1.5, 0.7, 0.4, 0.5), 0.5);
    }

    #[test]
    fn softness_has_a_smooth_transition() {
        let at_inner = vignette_factor(0.8, 0.0, 0.8, 0.5, 1.0);
        let halfway = vignette_factor(1.0, 0.0, 0.8, 0.5, 1.0);
        let at_outer = vignette_factor(1.3, 0.0, 0.8, 0.5, 1.0);
        close(at_inner, 1.0);
        assert!(halfway < 1.0 && halfway > 0.0);
        close(at_outer, 0.0);
    }

    #[test]
    fn alpha_is_preserved() {
        let mut pixels = [1.0, 0.8, 0.6, 0.37];
        apply_vignette(&mut pixels, 1, 1, 4, 3, true, 0.8, 0.2, 0.2);
        assert_eq!(pixels[3], 0.37);
    }

    #[test]
    fn transparent_pixels_are_unchanged() {
        let mut pixels = [0.8, 0.4, 0.2, 0.0];
        let before = pixels;
        apply_vignette(&mut pixels, 1, 1, 4, 3, true, 0.8, 0.2, 0.2);
        assert_eq!(pixels, before);
    }

    #[test]
    fn grayscale_is_supported() {
        let mut pixels = [1.0, 1.0, 1.0, 1.0];
        apply_vignette(&mut pixels, 2, 2, 1, 1, false, 1.0, 0.2, 0.1);
        assert!(pixels[0] < 1.0);
        assert!(pixels[3] < 1.0);
    }

    #[test]
    fn color_channels_change_but_alpha_does_not() {
        let mut pixels = [1.0, 0.8, 0.6, 0.5];
        apply_vignette(&mut pixels, 1, 1, 4, 3, true, 0.8, 0.2, 0.2);
        assert_eq!(pixels[3], 0.5);
        assert!(pixels[0] <= 1.0);
    }

    #[test]
    fn signed_parameter_parser_handles_negative_numbers() {
        assert_eq!(int_param(br#"{"strength":-5}"#, b"strength", 45), -5);
        assert_eq!(int_param(br#"{"radius":80}"#, b"radius", 70), 80);
    }
}
