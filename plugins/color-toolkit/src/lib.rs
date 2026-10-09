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
  "id":"org.photocraft.community.color-toolkit",
  "name":"Color Toolkit",
  "version":"0.1.0",
  "kind":"filter",
  "author":"ArtCraft Store community",
  "description":"Adjusts exposure, contrast, and saturation while preserving alpha.",
  "params":{
    "exposure":{"type":"int","min":-300,"max":300,"default":0},
    "contrast":{"type":"int","min":-100,"max":100,"default":0},
    "saturation":{"type":"int","min":0,"max":200,"default":100}
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
        let grown = core::arch::wasm32::memory_grow(0, pages_needed - pages_now);
        if grown == usize::MAX { return 0; }
    }
    unsafe { HEAP_NEXT = end; }
    start as i32
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub extern "C" fn pc_alloc(_size: i32) -> i32 { 0 }

#[derive(Clone, Copy, Debug, PartialEq)]
struct Color { r: f32, g: f32, b: f32 }

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

fn luminance(color: Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}

// Core-only approximation of 2^stops; avoids pulling a math library into WASM.
// The fractional part uses a degree-7 Taylor approximation of exp(x), x in [0, ln(2)].
fn exposure_gain(stops: f32) -> f32 {
    let mut whole = stops as i32;
    if stops < whole as f32 { whole -= 1; }
    let fraction = stops - whole as f32;
    let x = fraction * 0.69314718056;
    let mut exp_x = 1.0f32;
    let mut term = 1.0f32;
    let mut n = 1;
    while n <= 7 {
        term *= x / n as f32;
        exp_x += term;
        n += 1;
    }

    let mut gain = 1.0f32;
    if whole > 0 {
        let mut i = 0;
        while i < whole { gain *= 2.0; i += 1; }
    } else {
        let mut i = whole;
        while i < 0 { gain *= 0.5; i += 1; }
    }
    gain * exp_x
}

fn apply_color(color: Color, exposure: f32, contrast: f32, saturation: f32) -> Color {
    if exposure == 0.0 && contrast == 0.0 && saturation == 1.0 {
        return color;
    }
    let gain = exposure_gain(exposure);
    let mut result = Color {
        r: color.r * gain,
        g: color.g * gain,
        b: color.b * gain,
    };

    let factor = 1.0 + contrast;
    result.r = (result.r - 0.5) * factor + 0.5;
    result.g = (result.g - 0.5) * factor + 0.5;
    result.b = (result.b - 0.5) * factor + 0.5;

    let gray = luminance(result);
    Color {
        r: gray + (result.r - gray) * saturation,
        g: gray + (result.g - gray) * saturation,
        b: gray + (result.b - gray) * saturation,
    }
}

fn apply_pixel(pixel: &mut [f32], color_channels: usize, has_alpha: bool,
               exposure: f32, contrast: f32, saturation: f32) {
    if has_alpha && pixel[color_channels] <= 0.0 { return; }

    if color_channels == 3 {
        let color = Color { r: pixel[0], g: pixel[1], b: pixel[2] };
        let result = apply_color(color, exposure, contrast, saturation);
        pixel[0] = result.r;
        pixel[1] = result.g;
        pixel[2] = result.b;
    } else {
        // Saturation has no effect on grayscale/duotone data.
        let gain = exposure_gain(exposure);
        pixel[0] = (pixel[0] * gain - 0.5) * (1.0 + contrast) + 0.5;
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

    // ABI v1 stores Indexed documents as RGB; grayscale and duotone are one channel.
    let mode = (format >> 16) & 0xff;
    if !matches!(mode, 1 | 2 | 3 | 8) { return 5; }

    let has_alpha = (format & 0x100) != 0 && channels > 1;
    let color_channels = channels as usize - usize::from(has_alpha);
    if matches!(mode, 2 | 3) && color_channels != 3 { return 6; }
    if matches!(mode, 1 | 8) && color_channels != 1 { return 6; }

    let params = slice::from_raw_parts(params_ptr as *const u8, params_len as usize);
    let exposure = (int_param(params, b"exposure", 0).clamp(-300, 300) as f32) / 100.0;
    let contrast = (int_param(params, b"contrast", 0).clamp(-100, 100) as f32) / 100.0;
    let saturation = (int_param(params, b"saturation", 100).clamp(0, 200) as f32) / 100.0;

    if exposure == 0.0 && contrast == 0.0 && saturation == 1.0 { return 0; }

    let pixels = slice::from_raw_parts_mut(buf as *mut f32, count);
    let channels = channels as usize;
    for pixel in pixels.chunks_exact_mut(channels) {
        apply_pixel(pixel, color_channels, has_alpha, exposure, contrast, saturation);
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
    fn half_stop_exposure_uses_square_root_of_two() {
        close(exposure_gain(0.5), core::f32::consts::SQRT_2);
        close(exposure_gain(-0.5), 1.0 / core::f32::consts::SQRT_2);
    }

    #[test]
    fn exposure_zero_is_identity() {
        let c = Color { r: 0.2, g: 0.4, b: 0.8 };
        assert_eq!(apply_color(c, 0.0, 0.0, 1.0), c);
    }

    #[test]
    fn exposure_plus_one_stop_doubles_channels() {
        let c = apply_color(Color { r: 0.1, g: 0.2, b: 0.3 }, 1.0, 0.0, 1.0);
        close(c.r, 0.2);
        close(c.g, 0.4);
        close(c.b, 0.6);
    }

    #[test]
    fn contrast_zero_is_identity() {
        let c = Color { r: 0.15, g: 0.45, b: 0.9 };
        assert_eq!(apply_color(c, 0.0, 0.0, 1.0), c);
    }

    #[test]
    fn contrast_negative_one_flattens_to_mid_gray() {
        let c = apply_color(Color { r: 0.1, g: 0.4, b: 0.9 }, 0.0, -1.0, 1.0);
        close(c.r, 0.5);
        close(c.g, 0.5);
        close(c.b, 0.5);
    }

    #[test]
    fn saturation_zero_removes_color() {
        let c = apply_color(Color { r: 0.2, g: 0.4, b: 0.8 }, 0.0, 0.0, 0.0);
        close(c.r, c.g);
        close(c.g, c.b);
    }

    #[test]
    fn alpha_is_preserved() {
        let mut pixel = [0.2, 0.4, 0.6, 0.37];
        apply_pixel(&mut pixel, 3, true, 1.0, 0.1, 1.4);
        assert_eq!(pixel[3], 0.37);
    }

    #[test]
    fn transparent_pixel_is_unchanged() {
        let mut pixel = [0.2, 0.4, 0.6, 0.0];
        let before = pixel;
        apply_pixel(&mut pixel, 3, true, 1.0, 0.1, 1.4);
        assert_eq!(pixel, before);
    }

    #[test]
    fn signed_parameter_parser_reads_negative_values() {
        assert_eq!(int_param(br#"{"exposure":-250}"#, b"exposure", 0), -250);
        assert_eq!(int_param(br#"{"contrast":75}"#, b"contrast", 0), 75);
    }

    #[test]
    fn parameter_parser_uses_defaults_for_missing_keys() {
        assert_eq!(int_param(br#"{"contrast":20}"#, b"exposure", 0), 0);
        assert_eq!(int_param(br#"{"saturation":130}"#, b"saturation", 100), 130);
    }
}
