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
  "id":"org.photocraft.community.detail-recovery",
  "name":"Detail Recovery",
  "version":"0.1.0",
  "kind":"filter",
  "author":"ArtCraft Store community",
  "description":"Reduces small noise and restores local detail with lightweight spatial filtering.",
  "params":{
    "sharpen":{"type":"int","min":0,"max":100,"default":25},
    "denoise":{"type":"int","min":0,"max":100,"default":0}
  },
  "overlap":1,
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

fn filter_pixels(
    src: &[f32],
    dst: &mut [f32],
    width: usize,
    height: usize,
    channels: usize,
    color_channels: usize,
    has_alpha: bool,
    sharpen: f32,
    denoise: f32,
) {
    dst.copy_from_slice(src);
    let alpha_index = if has_alpha { Some(channels - 1) } else { None };

    for y in 0..height {
        for x in 0..width {
            let pixel = (y * width + x) * channels;
            if let Some(a) = alpha_index {
                if src[pixel + a] <= 0.0 { continue; }
            }

            let mut sums = [0.0f32; 3];
            let mut samples = 0.0f32;
            let y0 = y.saturating_sub(1);
            let y1 = (y + 1).min(height - 1);
            let x0 = x.saturating_sub(1);
            let x1 = (x + 1).min(width - 1);

            for ny in y0..=y1 {
                for nx in x0..=x1 {
                    let neighbor = (ny * width + nx) * channels;
                    if let Some(a) = alpha_index {
                        if src[neighbor + a] <= 0.0 { continue; }
                    }
                    for c in 0..color_channels {
                        sums[c] += src[neighbor + c];
                    }
                    samples += 1.0;
                }
            }

            if samples == 0.0 { continue; }
            for c in 0..color_channels {
                let original = src[pixel + c];
                let mean = sums[c] / samples;
                let smoothed = original + (mean - original) * denoise;
                // Add high-frequency detail back after denoising.
                dst[pixel + c] = smoothed + (original - mean) * sharpen;
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
    let sharpen = int_param(params, b"sharpen", 25).clamp(0, 100) as f32 / 100.0;
    let denoise = int_param(params, b"denoise", 0).clamp(0, 100) as f32 / 100.0;
    if sharpen == 0.0 && denoise == 0.0 { return 0; }

    let src_ptr = pc_alloc(buf_len);
    if src_ptr == 0 { return 4; }
    ptr::copy_nonoverlapping(buf as *const u8, src_ptr as *mut u8, buf_len as usize);
    let src = slice::from_raw_parts(src_ptr as *const f32, count);
    let dst = slice::from_raw_parts_mut(buf as *mut f32, count);
    filter_pixels(src, dst, width as usize, height as usize, ch, color_channels, has_alpha, sharpen, denoise);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.0001, "actual={actual}, expected={expected}");
    }

    #[test]
    fn zero_controls_are_identity() {
        let src = [0.0, 0.2, 0.8, 1.0];
        let mut dst = [0.0; 4];
        filter_pixels(&src, &mut dst, 2, 2, 1, 1, false, 0.0, 0.0);
        assert_eq!(src, dst);
    }

    #[test]
    fn denoise_moves_center_toward_local_mean() {
        let src = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let mut dst = [0.0; 9];
        filter_pixels(&src, &mut dst, 3, 3, 1, 1, false, 0.0, 1.0);
        close(dst[4], 1.0 / 9.0);
    }

    #[test]
    fn sharpening_pushes_center_away_from_local_mean() {
        let src = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let mut dst = [0.0; 9];
        filter_pixels(&src, &mut dst, 3, 3, 1, 1, false, 1.0, 0.0);
        close(dst[4], 1.0 + (1.0 - 1.0 / 9.0));
    }

    #[test]
    fn alpha_is_preserved() {
        let src = [0.0, 0.4, 0.0, 0.2, 0.8, 1.0, 0.4, 0.1];
        let mut dst = [0.0; 8];
        filter_pixels(&src, &mut dst, 2, 1, 4, 3, true, 1.0, 1.0);
        assert_eq!(dst[3], src[3]);
        assert_eq!(dst[7], src[7]);
    }

    #[test]
    fn fully_transparent_pixels_remain_unchanged() {
        let src = [0.2, 0.4, 0.6, 0.0, 0.8, 0.7, 0.6, 1.0];
        let mut dst = [0.0; 8];
        filter_pixels(&src, &mut dst, 2, 1, 4, 3, true, 1.0, 1.0);
        assert_eq!(&dst[0..4], &src[0..4]);
    }

    #[test]
    fn transparent_neighbors_do_not_bleed_into_opaque_pixels() {
        let src = [1.0, 0.0, 0.0, 0.0, 0.2, 0.2, 0.2, 1.0];
        let mut dst = [0.0; 8];
        filter_pixels(&src, &mut dst, 2, 1, 4, 3, true, 0.0, 1.0);
        close(dst[4], 0.2);
        close(dst[5], 0.2);
        close(dst[6], 0.2);
    }

    #[test]
    fn signed_and_missing_parameters_are_handled() {
        assert_eq!(int_param(br#"{"sharpen":72}"#, b"sharpen", 25), 72);
        assert_eq!(int_param(br#"{}"#, b"denoise", 0), 0);
    }
}
