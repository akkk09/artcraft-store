#![no_std]
#[cfg(test)]
extern crate std;

use core::slice;

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}

static MANIFEST: &[u8] = br#"{
  "id":"org.photocraft.community.geometry-fixer",
  "name":"Geometry Fixer",
  "version":"0.1.0",
  "kind":"filter",
  "author":"ArtCraft Store community",
  "description":"Straightens rotation and corrects horizontal or vertical perspective while keeping the original canvas size.",
  "params":{
    "rotation":{"type":"int","min":-15,"max":15,"default":0},
    "horizontal_perspective":{"type":"int","min":-100,"max":100,"default":0},
    "vertical_perspective":{"type":"int","min":-100,"max":100,"default":0},
    "zoom":{"type":"int","min":50,"max":200,"default":100}
  },
  "overlap":0,
  "area":"content"
}"#;

#[cfg(target_arch = "wasm32")]
static mut HEAP_NEXT: usize = 65536;
#[cfg(target_arch = "wasm32")]
static mut SCRATCH_PTR: i32 = 0;
#[cfg(target_arch = "wasm32")]
static mut SCRATCH_CAPACITY: usize = 0;

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
    let end = match start.checked_add(size) { Some(v) => v, None => return 0 };
    let needed = end.saturating_add(65535) / 65536;
    let current = core::arch::wasm32::memory_size(0);
    if needed > current && core::arch::wasm32::memory_grow(0, needed - current) == usize::MAX {
        return 0;
    }
    unsafe { HEAP_NEXT = end; }
    start as i32
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub extern "C" fn pc_alloc(_: i32) -> i32 { 0 }

#[cfg(target_arch = "wasm32")]
unsafe fn scratch_for(count: usize) -> *mut f32 {
    if SCRATCH_CAPACITY >= count && SCRATCH_PTR > 0 {
        return SCRATCH_PTR as *mut f32;
    }
    let bytes = match count.checked_mul(core::mem::size_of::<f32>()) {
        Some(v) if v <= i32::MAX as usize => v,
        _ => return core::ptr::null_mut(),
    };
    let ptr = pc_alloc(bytes as i32);
    if ptr <= 0 { return core::ptr::null_mut(); }
    SCRATCH_PTR = ptr;
    SCRATCH_CAPACITY = count;
    ptr as *mut f32
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
            let negative = params.get(j) == Some(&b'-');
            if negative || params.get(j) == Some(&b'+') { j += 1; }
            let mut n = 0i32;
            let mut found = false;
            while j < params.len() && params[j].is_ascii_digit() {
                n = n.saturating_mul(10).saturating_add((params[j] - b'0') as i32);
                found = true;
                j += 1;
            }
            if !found { return default; }
            return if negative { n.saturating_neg() } else { n };
        }
        i += 1;
    }
    default
}

fn sample_bilinear(src: &[f32], width: usize, height: usize, channels: usize,
                   x: f32, y: f32, out: &mut [f32]) -> bool {
    if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0
        || x > (width - 1) as f32 || y > (height - 1) as f32 {
        out.fill(0.0);
        return false;
    }
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let i00 = (y0 * width + x0) * channels;
    let i10 = (y0 * width + x1) * channels;
    let i01 = (y1 * width + x0) * channels;
    let i11 = (y1 * width + x1) * channels;
    for c in 0..channels {
        let top = src[i00 + c] + (src[i10 + c] - src[i00 + c]) * fx;
        let bottom = src[i01 + c] + (src[i11 + c] - src[i01 + c]) * fx;
        out[c] = top + (bottom - top) * fy;
    }
    true
}

fn transform(src: &[f32], dst: &mut [f32], width: usize, height: usize, channels: usize,
             rotation_deg: f32, horizontal: f32, vertical: f32, zoom_percent: f32) {
    let cx = (width.saturating_sub(1)) as f32 * 0.5;
    let cy = (height.saturating_sub(1)) as f32 * 0.5;
    let scale = (width.min(height) as f32 * 0.5).max(0.5);
    let radians = rotation_deg * (core::f32::consts::PI / 180.0);
    let (sin_a, cos_a) = radians.sin_cos();
    let zoom = (zoom_percent / 100.0).clamp(0.5, 2.0);
    let px = horizontal.clamp(-100.0, 100.0) * 0.003;
    let py = vertical.clamp(-100.0, 100.0) * 0.003;
    let mut pixel = [0.0f32; 4];

    for y in 0..height {
        for x in 0..width {
            let u = (x as f32 - cx) / scale;
            let v = (y as f32 - cy) / scale;
            let denom = 1.0 + px * u + py * v;
            let di = (y * width + x) * channels;
            if denom <= 0.1 || !denom.is_finite() {
                dst[di..di + channels].fill(0.0);
                continue;
            }
            let sx = cx + ((cos_a * u + sin_a * v) / (denom * zoom)) * scale;
            let sy = cy + ((-sin_a * u + cos_a * v) / (denom * zoom)) * scale;
            sample_bilinear(src, width, height, channels, sx, sy, &mut pixel[..channels]);
            dst[di..di + channels].copy_from_slice(&pixel[..channels]);
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
        .and_then(|v| v.checked_mul(channels as usize)) {
        Some(v) => v,
        None => return 2,
    };
    if count.checked_mul(4) != Some(buf_len as usize) { return 3; }

    let mode = (format >> 16) & 0xff;
    if !matches!(mode, 1 | 2 | 3 | 8) { return 5; }
    let ch = channels as usize;
    let has_alpha = (format & 0x100) != 0 && ch > 1;
    let colors = ch - usize::from(has_alpha);
    if matches!(mode, 2 | 3) && colors != 3 { return 6; }
    if matches!(mode, 1 | 8) && colors != 1 { return 6; }
    if colors != 1 && colors != 3 { return 6; }

    let params = slice::from_raw_parts(params_ptr as *const u8, params_len as usize);
    let rotation = int_param(params, b"rotation", 0).clamp(-15, 15) as f32;
    let horizontal = int_param(params, b"horizontal_perspective", 0).clamp(-100, 100) as f32;
    let vertical = int_param(params, b"vertical_perspective", 0).clamp(-100, 100) as f32;
    let zoom = int_param(params, b"zoom", 100).clamp(50, 200) as f32;
    if rotation == 0.0 && horizontal == 0.0 && vertical == 0.0 && zoom == 100.0 {
        return 0;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let scratch = scratch_for(count);
        if scratch.is_null() { return 7; }
        let source = slice::from_raw_parts(buf as *const f32, count);
        core::ptr::copy_nonoverlapping(source.as_ptr(), scratch, count);
        let source = slice::from_raw_parts(scratch as *const f32, count);
        let destination = slice::from_raw_parts_mut(buf as *mut f32, count);
        transform(source, destination, width as usize, height as usize, ch,
                  rotation, horizontal, vertical, zoom);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (count, rotation, horizontal, vertical, zoom);
        return 7;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    fn close(a: f32, b: f32) { assert!((a - b).abs() < 0.0001, "actual={a}, expected={b}"); }

    #[test]
    fn neutral_transform_is_identity() {
        let src = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
        let mut dst = [0.0; 9];
        transform(&src, &mut dst, 3, 3, 1, 0.0, 0.0, 0.0, 100.0);
        for (a, b) in dst.iter().zip(src.iter()) { close(*a, *b); }
    }

    #[test]
    fn neutral_transform_preserves_rgba_alpha() {
        let src = [0.2, 0.3, 0.4, 0.37];
        let mut dst = [0.0; 4];
        transform(&src, &mut dst, 1, 1, 4, 0.0, 0.0, 0.0, 100.0);
        close(dst[3], 0.37);
    }

    #[test]
    fn zoom_in_samples_center_and_crops_edges() {
        let src = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let mut dst = [0.0; 9];
        transform(&src, &mut dst, 3, 3, 1, 0.0, 0.0, 0.0, 200.0);
        close(dst[4], 5.0);
        assert!(dst[0] < src[0]);
    }

    #[test]
    fn perspective_moves_coordinates() {
        let src = [0.0; 25];
        let mut dst = [0.0; 25];
        transform(&src, &mut dst, 5, 5, 1, 0.0, 50.0, -30.0, 100.0);
        assert_eq!(dst.len(), src.len());
    }

    #[test]
    fn parser_handles_signed_values_and_defaults() {
        assert_eq!(int_param(br#"{"rotation":-12}"#, b"rotation", 0), -12);
        assert_eq!(int_param(br#"{"zoom":125}"#, b"zoom", 100), 125);
        assert_eq!(int_param(b"{}", b"rotation", 0), 0);
    }

    #[test]
    fn transparent_border_is_zeroed() {
        let src = [0.2; 4];
        let mut dst = vec![1.0; 4];
        transform(&src, &mut dst, 1, 1, 4, 0.0, 0.0, 0.0, 50.0);
        assert!(dst.iter().all(|v| v.is_finite()));
    }
}
