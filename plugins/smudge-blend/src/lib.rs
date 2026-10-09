#![no_std]

use core::{ptr, slice};

static MANIFEST: &[u8] = br#"{
  "id":"org.photocraft.community.smudge-blend",
  "name":"Smudge Blend",
  "version":"0.1.0",
  "kind":"filter",
  "author":"PhotoCraft Plugin Store",
  "description":"A directional smudge-inspired filter for the active layer. This is a whole-layer filter, not a cursor-driven brush.",
  "params":{
    "strength":{"type":"int","min":0,"max":100,"default":55},
    "radius":{"type":"int","min":1,"max":16,"default":4},
    "direction":{"type":"choice","options":["right","left","down","up","down-right","down-left"],"default":"right"}
  },
  "overlap":16,
  "area":"content"
}"#;

static mut HEAP_NEXT: usize = 65536;

#[no_mangle]
pub extern "C" fn pc_abi_version() -> i32 { 1 }

#[no_mangle]
pub extern "C" fn pc_manifest() -> i64 {
    (((MANIFEST.len() as i64) << 32) | (MANIFEST.as_ptr() as u32 as i64))
}

#[no_mangle]
pub extern "C" fn pc_alloc(size: i32) -> i32 {
    if size <= 0 { return 0; }
    let align = 8usize;
    let size = size as usize;
    let start = unsafe { (HEAP_NEXT + align - 1) & !(align - 1) };
    let end = match start.checked_add(size) { Some(v) => v, None => return 0 };
    let pages_needed = (end + 65535) / 65536;
    let pages_now = core::arch::wasm32::memory_size(0);
    if pages_needed > pages_now {
        let delta = pages_needed - pages_now;
        let grown = core::arch::wasm32::memory_grow(0, delta);
        if grown == usize::MAX { return 0; }
    }
    unsafe { HEAP_NEXT = end; }
    start as i32
}

fn int_param(params: &[u8], key: &[u8], default: i32) -> i32 {
    let mut i = 0;
    while i + key.len() < params.len() {
        if params[i] == b'"' && params.get(i + 1..i + 1 + key.len()) == Some(key) &&
           params.get(i + 1 + key.len()) == Some(&b'"') {
            let mut j = i + key.len() + 2;
            while j < params.len() && (params[j] == b' ' || params[j] == b'\t' || params[j] == b'\n' || params[j] == b'\r') { j += 1; }
            if params.get(j) != Some(&b':') { return default; }
            j += 1;
            while j < params.len() && params[j].is_ascii_whitespace() { j += 1; }
            let mut n = 0i32;
            let mut found = false;
            while j < params.len() && params[j].is_ascii_digit() {
                n = n.saturating_mul(10).saturating_add((params[j] - b'0') as i32);
                found = true; j += 1;
            }
            if found { return n; }
            return default;
        }
        i += 1;
    }
    default
}

fn direction(params: &[u8]) -> (isize, isize) {
    let mut i = 0;
    while i + 11 < params.len() {
        if params[i..].starts_with(b"\"direction\"") {
            let rest = &params[i + 11..];
            if let Some(colon) = rest.iter().position(|&b| b == b':') {
                let value = &rest[colon + 1..];
                if value.windows(4).any(|w| w == b"left") { return (-1, 0); }
                if value.windows(4).any(|w| w == b"down") {
                    if value.windows(10).any(|w| w == b"down-left") { return (-1, 1); }
                    if value.windows(11).any(|w| w == b"down-right") { return (1, 1); }
                    return (0, 1);
                }
                if value.windows(2).any(|w| w == b"up") { return (0, -1); }
            }
        }
        i += 1;
    }
    (1, 0)
}

#[no_mangle]
pub unsafe extern "C" fn pc_filter(
    buf: i32, buf_len: i32, width: i32, height: i32, channels: i32,
    format: i32, params_ptr: i32, params_len: i32
) -> i32 {
    if buf <= 0 || width <= 0 || height <= 0 || channels <= 0 || params_ptr < 0 || params_len < 0 { return 1; }
    let count = match (width as usize).checked_mul(height as usize).and_then(|v| v.checked_mul(channels as usize)) { Some(v) => v, None => return 2 };
    if count.checked_mul(4) != Some(buf_len as usize) { return 3; }
    let params = slice::from_raw_parts(params_ptr as *const u8, params_len as usize);
    let strength = int_param(params, b"strength", 55).clamp(0, 100) as f32 / 100.0;
    let radius = int_param(params, b"radius", 4).clamp(1, 16) as isize;
    let (dx, dy) = direction(params);
    let src_ptr = pc_alloc(buf_len);
    if src_ptr == 0 { return 4; }
    ptr::copy_nonoverlapping(buf as *const u8, src_ptr as *mut u8, buf_len as usize);
    let src = slice::from_raw_parts(src_ptr as *const f32, count);
    let dst = slice::from_raw_parts_mut(buf as *mut f32, count);
    let w = width as isize; let h = height as isize; let ch = channels as usize;
    let has_alpha = (format & 0x100) != 0 && ch > 1;
    let color_ch = if has_alpha { ch - 1 } else { ch };
    for y in 0..h {
        for x in 0..w {
            let p = ((y * w + x) as usize) * ch;
            if has_alpha && src[p + ch - 1] <= 0.0 { continue; }
            let sx = (x - dx * radius).clamp(0, w - 1);
            let sy = (y - dy * radius).clamp(0, h - 1);
            let q = ((sy * w + sx) as usize) * ch;
            for c in 0..color_ch {
                let dragged = src[q + c] * 0.65 + src[((y - dy).clamp(0,h-1) * w + (x - dx).clamp(0,w-1)) as usize * ch + c] * 0.35;
                dst[p + c] = src[p + c] * (1.0 - strength) + dragged * strength;
            }
        }
    }
    0
}