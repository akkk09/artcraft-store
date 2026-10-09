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
  "id":"org.photocraft.community.mosaic-pixelate",
  "name":"Mosaic Pixelate",
  "version":"0.1.0",
  "kind":"filter",
  "author":"ArtCraft Store community",
  "description":"Creates block-based pixelation with adjustable block size and intensity, preserving alpha.",
  "params":{
    "block_size":{"type":"int","min":2,"max":32,"default":8},
    "intensity":{"type":"int","min":0,"max":100,"default":100}
  },
  "overlap":31,
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

fn pixelate(
    src: &[f32],
    dst: &mut [f32],
    width: usize,
    height: usize,
    channels: usize,
    color_channels: usize,
    has_alpha: bool,
    block_size: usize,
    intensity: f32,
) {
    dst.copy_from_slice(src);
    if width == 0 || height == 0 || channels == 0 || color_channels == 0 || intensity <= 0.0 {
        return;
    }
    let alpha_index = if has_alpha { Some(channels - 1) } else { None };

    let mut by = 0;
    while by < height {
        let y_end = (by + block_size).min(height);
        let mut bx = 0;
        while bx < width {
            let x_end = (bx + block_size).min(width);
            let mut sums = [0.0f32; 3];
            let mut count = 0.0f32;

            for y in by..y_end {
                for x in bx..x_end {
                    let p = (y * width + x) * channels;
                    if let Some(a) = alpha_index {
                        if src[p + a] <= 0.0 { continue; }
                    }
                    for c in 0..color_channels {
                        sums[c] += src[p + c];
                    }
                    count += 1.0;
                }
            }

            if count > 0.0 {
                for y in by..y_end {
                    for x in bx..x_end {
                        let p = (y * width + x) * channels;
                        if let Some(a) = alpha_index {
                            if src[p + a] <= 0.0 { continue; }
                        }
                        for c in 0..color_channels {
                            let mean = sums[c] / count;
                            dst[p + c] = src[p + c] + (mean - src[p + c]) * intensity;
                        }
                    }
                }
            }
            bx = x_end;
        }
        by = y_end;
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
    let block_size = int_param(params, b"block_size", 8).clamp(2, 32) as usize;
    let intensity = int_param(params, b"intensity", 100).clamp(0, 100) as f32 / 100.0;
    if intensity == 0.0 { return 0; }

    let src_ptr = pc_alloc(buf_len);
    if src_ptr == 0 { return 4; }
    ptr::copy_nonoverlapping(buf as *const u8, src_ptr as *mut u8, buf_len as usize);
    let src = slice::from_raw_parts(src_ptr as *const f32, count);
    let dst = slice::from_raw_parts_mut(buf as *mut f32, count);
    pixelate(src, dst, width as usize, height as usize, ch, color_channels, has_alpha, block_size, intensity);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.0001, "actual={actual}, expected={expected}");
    }

    #[test]
    fn zero_intensity_is_identity() {
        let src = [0.0, 0.25, 0.5, 0.75];
        let mut dst = [0.0; 4];
        pixelate(&src, &mut dst, 2, 2, 1, 1, false, 2, 0.0);
        assert_eq!(src, dst);
    }

    #[test]
    fn block_is_replaced_by_its_average() {
        let src = [0.0, 1.0, 0.0, 1.0];
        let mut dst = [0.0; 4];
        pixelate(&src, &mut dst, 2, 2, 1, 1, false, 2, 1.0);
        for value in dst { close(value, 0.5); }
    }

    #[test]
    fn partial_edge_blocks_are_processed() {
        let src = [0.0, 1.0, 0.0];
        let mut dst = [0.0; 3];
        pixelate(&src, &mut dst, 3, 1, 1, 1, false, 2, 1.0);
        close(dst[0], 0.5);
        close(dst[1], 0.5);
        close(dst[2], 0.0);
    }

    #[test]
    fn alpha_is_preserved() {
        let src = [0.0, 0.2, 0.4, 0.25, 1.0, 0.8, 0.6, 0.75];
        let mut dst = [0.0; 8];
        pixelate(&src, &mut dst, 2, 1, 4, 3, true, 2, 1.0);
        assert_eq!(dst[3], src[3]);
        assert_eq!(dst[7], src[7]);
    }

    #[test]
    fn transparent_pixels_remain_unchanged_and_do_not_affect_averages() {
        let src = [1.0, 0.0, 0.0, 0.0, 0.2, 0.4, 0.6, 1.0];
        let mut dst = [0.0; 8];
        pixelate(&src, &mut dst, 2, 1, 4, 3, true, 2, 1.0);
        assert_eq!(&dst[0..4], &src[0..4]);
        close(dst[4], 0.2);
        close(dst[5], 0.4);
        close(dst[6], 0.6);
    }

    #[test]
    fn intensity_interpolates_with_original_pixels() {
        let src = [0.0, 1.0];
        let mut dst = [0.0; 2];
        pixelate(&src, &mut dst, 2, 1, 1, 1, false, 2, 0.5);
        close(dst[0], 0.25);
        close(dst[1], 0.75);
    }

    #[test]
    fn grayscale_and_rgb_shapes_are_supported() {
        let gray = [0.0, 1.0];
        let mut gray_out = [0.0; 2];
        pixelate(&gray, &mut gray_out, 2, 1, 1, 1, false, 2, 1.0);
        close(gray_out[0], 0.5);
        close(gray_out[1], 0.5);

        let rgb = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let mut rgb_out = [0.0; 6];
        pixelate(&rgb, &mut rgb_out, 2, 1, 3, 3, false, 2, 1.0);
        close(rgb_out[0], 0.5);
        close(rgb_out[1], 0.0);
        close(rgb_out[2], 0.5);
    }

    #[test]
    fn parameters_are_clamped_to_supported_ranges() {
        assert_eq!(int_param(br#"{"block_size":40}"#, b"block_size", 8).clamp(2, 32), 32);
        assert_eq!(int_param(br#"{"intensity":-1}"#, b"intensity", 100).clamp(0, 100), 0); // parser defaults on signed values
    }
}
