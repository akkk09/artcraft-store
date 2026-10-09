#![no_std]

use core::{ptr, slice};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}


static MANIFEST: &[u8] = br#"{
  "id":"org.photocraft.community.mixer-blend",
  "name":"Mixer Blend",
  "version":"0.1.0",
  "kind":"filter",
  "author":"PhotoCraft Plugin Store",
  "description":"A wet-paint-inspired colour mixing filter with pickup and wetness controls. This is not an interactive brush.",
  "params":{
    "strength":{"type":"int","min":0,"max":100,"default":45},
    "radius":{"type":"int","min":1,"max":12,"default":3},
    "wetness":{"type":"int","min":0,"max":100,"default":65},
    "pickup":{"type":"int","min":0,"max":100,"default":50}
  },
  "overlap":12,
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
    let size = size as usize;
    let start = unsafe { (HEAP_NEXT + 7) & !7usize };
    let end = match start.checked_add(size) { Some(v) => v, None => return 0 };
    let needed = (end + 65535) / 65536;
    let current = core::arch::wasm32::memory_size(0);
    if needed > current && core::arch::wasm32::memory_grow(0, needed - current) == usize::MAX { return 0; }
    unsafe { HEAP_NEXT = end; }
    start as i32
}

fn param(params: &[u8], key: &[u8], default: i32) -> i32 {
    let mut i = 0;
    while i + key.len() + 2 < params.len() {
        if params[i] == b'"' && params.get(i + 1..i + 1 + key.len()) == Some(key) && params.get(i + key.len() + 1) == Some(&b'"') {
            let mut j = i + key.len() + 2;
            while j < params.len() && params[j].is_ascii_whitespace() { j += 1; }
            if params.get(j) != Some(&b':') { return default; }
            j += 1;
            while j < params.len() && params[j].is_ascii_whitespace() { j += 1; }
            let mut n = 0i32; let mut found = false;
            while j < params.len() && params[j].is_ascii_digit() { n = n.saturating_mul(10).saturating_add((params[j]-b'0') as i32); found = true; j += 1; }
            return if found { n } else { default };
        }
        i += 1;
    }
    default
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
    let strength = param(params, b"strength", 45).clamp(0, 100) as f32 / 100.0;
    let radius = param(params, b"radius", 3).clamp(1, 12) as isize;
    let wet = param(params, b"wetness", 65).clamp(0, 100) as f32 / 100.0;
    let pickup = param(params, b"pickup", 50).clamp(0, 100) as f32 / 100.0;
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
            let p = ((y*w+x) as usize)*ch;
            if has_alpha && src[p+ch-1] <= 0.0 { continue; }
            let mut sums = [0.0f32; 8];
            let mut n = 0.0f32;
            for oy in -radius..=radius {
                for ox in -radius..=radius {
                    if ox*ox + oy*oy > radius*radius { continue; }
                    let xx = (x+ox).clamp(0,w-1); let yy = (y+oy).clamp(0,h-1);
                    let q = ((yy*w+xx) as usize)*ch;
                    for c in 0..color_ch.min(8) { sums[c] += src[q+c]; }
                    n += 1.0;
                }
            }
            if n <= 0.0 { continue; }
            for c in 0..color_ch.min(8) {
                let local = sums[c] / n;
                let picked = src[p+c] * (1.0-pickup) + local * pickup;
                let paint_mix = src[p+c] * (1.0-wet) + picked * wet;
                dst[p+c] = src[p+c] * (1.0-strength) + paint_mix * strength;
            }
        }
    }
    0
}