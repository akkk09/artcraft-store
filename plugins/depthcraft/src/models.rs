//! Openly-licensed monocular depth estimation model registry, license documentation, and verification.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceSelection {
    Cpu,
    Gpu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityPreset {
    Fast,     // 256x256 inference resolution
    Balanced, // 384x384 inference resolution
    Ultra,    // 512x512 inference resolution
}

impl QualityPreset {
    pub fn resolution(&self) -> (usize, usize) {
        match self {
            QualityPreset::Fast => (256, 256),
            QualityPreset::Balanced => (384, 384),
            QualityPreset::Ultra => (512, 512),
        }
    }
}

/// Metadata describing a supported depth estimation model.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub authors: &'static str,
    pub license: &'static str,
    pub license_url: &'static str,
    pub homepage: &'static str,
    pub default_resolution: (usize, usize),
    pub weights_file: &'static str,
    pub sha256: &'static str,
}

/// The registry of supported depth estimation models.
pub const MODELS: &[ModelSpec] = &[
    ModelSpec {
        id: "classical",
        name: "Classical Gradient & Focus Proxy",
        description: "Pure-Rust multi-scale edge gradient and focus depth estimator. Zero external weights, deterministic, runs in sandboxed wasm and CPU.",
        authors: "EffectCraft contributors",
        license: "MIT OR Apache-2.0",
        license_url: "https://github.com/storytold/effectcraft/blob/main/LICENSE-MIT",
        homepage: "https://github.com/storytold/effectcraft",
        default_resolution: (256, 256),
        weights_file: "embedded",
        sha256: "builtin",
    },
    ModelSpec {
        id: "midas_v21_small",
        name: "MiDaS v2.1 Small",
        description: "Efficient monocular depth estimation network (René Ranftl et al., Intel ISL, 2021). Optimized for fast edge and relative depth recovery.",
        authors: "René Ranftl, Katrin Lasinger, David Hafner, Konrad Schindler, Vladlen Koltun (Intel ISL)",
        license: "MIT",
        license_url: "https://github.com/isl-org/MiDaS/blob/master/LICENSE",
        homepage: "https://github.com/isl-org/MiDaS",
        default_resolution: (256, 256),
        weights_file: "midas_v21_small.tflite",
        sha256: "8e98da69b0faee2496a8435d8bc1b29a656ea9c20a8f88cf50fca1b942079075",
    },
    ModelSpec {
        id: "depth_anything_v2_small",
        name: "Depth Anything V2 Small",
        description: "Vision Transformer-based monocular depth estimation model trained on synthetic and large-scale real data (Lihe Yang et al., 2024).",
        authors: "Lihe Yang, Bingyi Kang, Zilong Huang, Xiaogang Xu, Jiashi Feng, Hengshuang Zhao (HKU & TikTok)",
        license: "Apache-2.0",
        license_url: "https://github.com/DepthAnything/Depth-Anything-V2/blob/main/LICENSE",
        homepage: "https://github.com/DepthAnything/Depth-Anything-V2",
        default_resolution: (518, 518),
        weights_file: "depth_anything_v2_vits.tflite",
        sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    },
];

pub fn find_model(id: &str) -> Option<&'static ModelSpec> {
    MODELS.iter().find(|m| m.id == id)
}

/// Verify that an external weights buffer matches the model's pinned SHA-256.
pub fn verify_weights(model: &ModelSpec, bytes: &[u8]) -> Result<(), String> {
    if model.sha256 == "builtin" {
        return Ok(());
    }
    let calculated = sha256_hex(bytes);
    if calculated != model.sha256 {
        return Err(format!("model `{}` checksum mismatch: expected {}, got {}", model.id, model.sha256, calculated));
    }
    Ok(())
}

/// Simple SHA-256 digest function (FIPS 180-4).
pub fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let bits = (data.len() as u64).wrapping_mul(8);
    let mut tail = data[data.len() - data.len() % 64..].to_vec();
    tail.push(0x80);
    while tail.len() % 64 != 56 {
        tail.push(0);
    }
    tail.extend_from_slice(&bits.to_be_bytes());

    for block in data[..data.len() - data.len() % 64].as_chunks::<64>().0.iter().chain(tail.as_chunks::<64>().0.iter()) {
        let mut w = [0u32; 64];
        for (i, c) in block.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(y);
        }
    }

    let mut out = String::with_capacity(64);
    for v in h {
        out.push_str(&format!("{v:08x}"));
    }
    out
}
