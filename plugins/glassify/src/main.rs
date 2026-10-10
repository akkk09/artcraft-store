//! Glassify CLI benchmark and testing utility.

use std::time::Instant;
use glassify::{MANIFEST, default_params, render_frame};
use glassify::presets::{PresetConfig, PresetId};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("bench");

    match mode {
        "manifest" => {
            println!("{}", MANIFEST);
        }
        "presets" => {
            println!("Available Glassify Presets:");
            for (idx, name) in [
                (1, "Lucid"),
                (2, "Frosted / Satin"),
                (3, "Liquid Glass"),
                (4, "Onyx Smoke"),
                (5, "Prismatic Crystal"),
                (6, "Ribbed / Fluted"),
            ] {
                let cfg = PresetConfig::for_preset(PresetId::from_index(idx)).unwrap();
                println!("  [{}] {}: IOR={:.2}, thickness={:.1}, roughness={:.1}, dispersion={:.1}",
                    idx, name, cfg.ior, cfg.thickness, cfg.roughness, cfg.chromatic_dispersion);
            }
        }
        "bench" | _ => {
            println!("Running Glassify Performance Benchmark...");
            let width = 1920;
            let height = 1080;
            let total = width * height;

            // Generate synthetic transparent logo/card test pattern
            let mut pixels = vec![[0.0f32; 4]; total];
            let cx = width as f32 * 0.5;
            let cy = height as f32 * 0.5;

            for y in 0..height {
                for x in 0..width {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    // Rounded glass badge in center
                    let in_box = dx.abs() < 400.0 && dy.abs() < 250.0;
                    if in_box {
                        pixels[y * width + x] = [0.2, 0.4, 0.8, 1.0];
                    } else {
                        // Background gradient
                        let bg = ((x as f32 / width as f32) * 0.8).clamp(0.0, 1.0);
                        pixels[y * width + x] = [bg, bg * 0.5, 0.2, 1.0];
                    }
                }
            }

            let mut params = default_params();
            // Test Lucid preset
            params[0] = 1.0;

            let start = Instant::now();
            let res = render_frame(&mut pixels, width, height, &params, 0.0, 1.0);
            let elapsed = start.elapsed();

            match res {
                Ok(_) => {
                    println!("Rendered 1080p frame ({}x{}) successfully in {:.2?}", width, height, elapsed);
                    println!("FPS equivalent: {:.1} fps", 1.0 / elapsed.as_secs_f64());
                }
                Err(e) => {
                    eprintln!("Render failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
