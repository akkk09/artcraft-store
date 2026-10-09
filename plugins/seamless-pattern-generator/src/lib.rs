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
  "id":"org.photocraft.community.seamless-pattern-generator",
  "name":"Seamless Pattern Generator",
  "version":"0.1.0",
  "kind":"filter",
  "author":"ArtCraft Store community",
  "description":"Reduces visible tiling seams by smoothly blending opposing edges.",
  "params":{
    "strength":{"type":"int","min":0,"max":100,"default":100},
    "edge_width":{"type":"int","min":1,"max":50,"default":20}
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

fn smooth_edge_weight(distance: usize, band: usize, strength: f32) -> f32 {
    if band == 0 || distance >= band || strength <= 0.0 { return 0.0; }
    let t = 1.0 - distance as f32 / band as f32;
    strength * t * t * (3.0 - 2.0 * t)
}

fn edge_weight(position: usize, length: usize, band: usize, strength: f32) -> f32 {
    let opposite_distance = length - 1 - position;
    let distance = position.min(opposite_distance);
    smooth_edge_weight(distance, band, strength)
}

fn apply_seamless(
    src: &[f32],
    dst: &mut [f32],
    width: usize,
    height: usize,
    channels: usize,
    color_channels: usize,
    strength: f32,
    edge_percent: usize,
) {
    if width == 0 || height == 0 || channels == 0 || strength <= 0.0 { return; }
    let band_x = ((width * edge_percent) / 100).max(1).min(width / 2 + width % 2);
    let band_y = ((height * edge_percent) / 100).max(1).min(height / 2 + height % 2);

    for y in 0..height {
        let opposite_y = height - 1 - y;
        let wy = edge_weight(y, height, band_y, strength);
        for x in 0..width {
            let opposite_x = width - 1 - x;
            let wx = edge_weight(x, width, band_x, strength);
            let weight = wx.max(wy);
            let dst_i = (y * width + x) * channels;
            let src_i = dst_i;
            if weight <= 0.0 {
                dst[dst_i..dst_i + channels].copy_from_slice(&src[src_i..src_i + channels]);
                continue;
            }

            let horizontal = wx > 0.0;
            let vertical = wy > 0.0;
            let mut total = 0.0f32;
            for c in 0..color_channels {
                let mut sum = src[src_i + c];
                let mut samples = 1.0f32;
                if horizontal {
                    sum += src[(y * width + opposite_x) * channels + c];
                    samples += 1.0;
                }
                if vertical {
                    sum += src[(opposite_y * width + x) * channels + c];
                    samples += 1.0;
                }
                if horizontal && vertical {
                    sum += src[(opposite_y * width + opposite_x) * channels + c];
                    samples += 1.0;
                }
                total = sum / samples;
                dst[dst_i + c] = src[src_i + c] + (total - src[src_i + c]) * weight;
            }
            if color_channels < channels {
                dst[dst_i + channels - 1] = src[src_i + channels - 1];
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
    if color_channels != 1 && color_channels != 3 { return 6; }

    let params = slice::from_raw_parts(params_ptr as *const u8, params_len as usize);
    let strength = int_param(params, b"strength", 100).clamp(0, 100) as f32 / 100.0;
    let edge_width = int_param(params, b"edge_width", 20).clamp(1, 50) as usize;
    if strength <= 0.0 { return 0; }

    #[cfg(target_arch = "wasm32")]
    {
        let scratch = scratch_for(count);
        if scratch.is_null() { return 7; }
        let source = slice::from_raw_parts(buf as *const f32, count);
        core::ptr::copy_nonoverlapping(source.as_ptr(), scratch, count);
        let source = slice::from_raw_parts(scratch as *const f32, count);
        let destination = slice::from_raw_parts_mut(buf as *mut f32, count);
        apply_seamless(source, destination, width as usize, height as usize, ch,
                       color_channels, strength, edge_width);
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (count, strength, edge_width);
        7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.0001, "actual={actual}, expected={expected}");
    }

    #[test]
    fn zero_strength_preserves_input() {
        let src = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5];
        let mut dst = [9.0; 6];
        apply_seamless(&src, &mut dst, 3, 2, 1, 1, 0.0, 20);
        assert_eq!(dst, src);
    }

    #[test]
    fn horizontal_opposite_edges_match_at_full_strength() {
        let src = [0.0, 0.2, 0.8, 1.0, 0.1, 0.3, 0.7, 0.9];
        let mut dst = [0.0; 8];
        apply_seamless(&src, &mut dst, 4, 2, 1, 1, 1.0, 25);
        close(dst[0], dst[3]);
        close(dst[4], dst[7]);
    }

    #[test]
    fn vertical_opposite_edges_match_at_full_strength() {
        let src = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
        let mut dst = [0.0; 6];
        apply_seamless(&src, &mut dst, 3, 2, 1, 1, 1.0, 50);
        for x in 0..3 { close(dst[x], dst[3 + x]); }
    }

    #[test]
    fn alpha_is_preserved() {
        let src = [0.0, 0.2, 0.4, 0.17, 0.6, 0.8, 1.0, 0.73];
        let mut dst = [0.0; 8];
        apply_seamless(&src, &mut dst, 2, 1, 4, 3, 1.0, 50);
        close(dst[3], 0.17);
        close(dst[7], 0.73);
    }

    #[test]
    fn blend_fades_toward_interior() {
        let src = [0.0, 0.5, 1.0, 0.5, 0.0];
        let mut dst = [0.0; 5];
        apply_seamless(&src, &mut dst, 5, 1, 1, 1, 1.0, 20);
        close(dst[0], dst[4]);
        close(dst[2], src[2]);
    }

    #[test]
    fn parser_handles_values_and_defaults() {
        assert_eq!(int_param(br#"{"strength":85}"#, b"strength", 100), 85);
        assert_eq!(int_param(br#"{"edge_width":12}"#, b"edge_width", 20), 12);
        assert_eq!(int_param(b"{}", b"strength", 100), 100);
    }
}
