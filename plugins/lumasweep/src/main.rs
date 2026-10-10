//! lumasweep-cli: Command line interface and benchmark utility for LumaSweep.

use std::time::Instant;

use lumasweep::presets::{PresetConfig, PresetId};
use lumasweep::render_frame;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--presets" || a == "presets") {
        print_presets();
        return;
    }

    if args.iter().any(|a| a == "--bench" || a == "bench") {
        run_benchmark();
        return;
    }

    if args.iter().any(|a| a == "--demo" || a == "demo") {
        run_demo_render();
        return;
    }

    println!("LumaSweep CLI - Controllable Light Sweep for EffectCraft");
    println!("Usage:");
    println!("  lumasweep-cli --presets    List built-in presets and configurations");
    println!("  lumasweep-cli --bench      Run performance benchmarks on synthetic layers");
    println!("  lumasweep-cli --demo       Generate demonstration PPM test images");
}

fn print_presets() {
    println!("LumaSweep Built-in Presets:");
    println!("---------------------------");
    for id in [
        PresetId::Chrome,
        PresetId::SoftStudio,
        PresetId::Prism,
        PresetId::BrushedMetal,
        PresetId::GoldLustre,
        PresetId::LaserBeam,
    ] {
        if let Some(cfg) = PresetConfig::for_preset(id) {
            println!("{:?}:", id);
            println!("  Width: {:.1} px | Softness: {:.1}% | Sweep Intensity: {:.1}%", cfg.width, cfg.softness, cfg.sweep_intensity);
            println!("  Highlight: RGBA({:.2}, {:.2}, {:.2}, {:.2})", cfg.highlight_color[0], cfg.highlight_color[1], cfg.highlight_color[2], cfg.highlight_color[3]);
            println!("  Bevel: Intensity {:.1}% | Depth {:.1} px | Profile {}", cfg.bevel_intensity, cfg.bevel_depth, cfg.bevel_profile);
            println!("  Optics: Fringe {:.1}% | Glow {:.1}% (R={:.1}px) | Texture {:.1}%", cfg.chromatic_fringe, cfg.glow_intensity, cfg.glow_radius, cfg.texture_intensity);
            println!();
        }
    }
}

fn run_benchmark() {
    println!("Running LumaSweep Performance Benchmark...");
    println!("-------------------------------------------");

    let width = 1280;
    let height = 720;
    let total_pixels = width * height;
    let frame_count = 30;

    println!("Allocating 720p HD frame ({width}x{height}, {total_pixels} pixels)...");

    // Synthetic text/logo mask with circular graphic and text band
    let mut pixels = vec![[0.0f32; 4]; total_pixels];
    for y in 0..height {
        let ny = (y as f32 / height as f32) - 0.5;
        for x in 0..width {
            let nx = (x as f32 / width as f32) - 0.5;
            let dist_center = (nx * nx + ny * ny).sqrt();

            // Text / logo shape: circular ring with cross bar
            let in_ring = dist_center > 0.15 && dist_center < 0.28;
            let in_bar = ny.abs() < 0.05 && nx.abs() < 0.35;
            let a = if in_ring || in_bar { 1.0f32 } else { 0.0f32 };
            let r = 0.85 * a;
            let g = 0.85 * a;
            let b = 0.90 * a;
            pixels[y * width + x] = [r, g, b, a];
        }
    }

    // Default parameters with auto-animate enabled
    let mut params = vec![0.0f64; 32];
    params[0] = 1.0; // Chrome preset
    params[1] = 0.5; // center x
    params[2] = 0.5; // center y
    params[3] = -35.0; // direction angle
    params[28] = 1.0; // auto-animate on
    params[29] = 1.0; // speed

    let start = Instant::now();
    let mut scratch = pixels.clone();

    for f in 0..frame_count {
        scratch.copy_from_slice(&pixels);
        let time = f as f64 * (1.0 / 30.0);
        render_frame(&mut scratch, width, height, &params, time, 1.0).expect("render failed");
    }

    let elapsed = start.elapsed();
    let fps = frame_count as f64 / elapsed.as_secs_f64();
    let ms_per_frame = (elapsed.as_secs_f64() * 1000.0) / frame_count as f64;

    println!("Completed {frame_count} 720p frames in {:.3}s ({:.1} FPS, {:.2} ms/frame)", elapsed.as_secs_f64(), fps, ms_per_frame);
}

fn run_demo_render() {
    println!("Generating LumaSweep demo frames...");
    let width = 400;
    let height = 200;
    let mut pixels = vec![[0.0f32; 4]; width * height];

    for y in 0..height {
        let ny = (y as f32 / height as f32) - 0.5;
        for x in 0..width {
            let nx = (x as f32 / width as f32) - 0.5;
            let in_text_box = ny.abs() < 0.25 && nx.abs() < 0.4;
            let a = if in_text_box { 1.0f32 } else { 0.0f32 };
            pixels[y * width + x] = [0.8 * a, 0.8 * a, 0.8 * a, a];
        }
    }

    let mut params = vec![0.0f64; 32];
    params[0] = 3.0; // Prism preset
    params[1] = 0.5;
    params[2] = 0.5;

    render_frame(&mut pixels, width, height, &params, 0.0, 1.0).expect("demo render failed");
    println!("Demo frame rendered successfully (400x200 Prism preset).");
}
