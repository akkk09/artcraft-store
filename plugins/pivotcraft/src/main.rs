//! PivotCraft CLI utility for anchor point math, benchmarks, and preset querying.

use std::env;
use std::time::Instant;
use pivotcraft::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") || args.len() <= 1 {
        println!("PivotCraft CLI - Anchor Point Placement & Transform Compensation Engine");
        println!("Usage:");
        println!("  pivotcraft-cli --bench              Benchmark 2D & 3D transform math solvers");
        println!("  pivotcraft-cli --presets            List built-in presets as JSON");
        println!("  pivotcraft-cli --solve-2d <dx> <dy> <sx> <sy> <rot_deg>");
        println!("  pivotcraft-cli --solve-3d <dx> <dy> <dz> <sx> <sy> <sz> <rx> <ry> <rz>");
        return;
    }

    if args.iter().any(|a| a == "--bench") {
        run_benchmarks();
        return;
    }

    if args.iter().any(|a| a == "--presets") {
        let presets = built_in_presets();
        println!("{}", serde_json::to_string_pretty(&presets).unwrap());
        return;
    }

    if let Some(pos) = args.iter().position(|a| a == "--solve-2d") {
        if args.len() >= pos + 6 {
            let dx: f64 = args[pos + 1].parse().unwrap_or(0.0);
            let dy: f64 = args[pos + 2].parse().unwrap_or(0.0);
            let sx: f64 = args[pos + 3].parse().unwrap_or(100.0);
            let sy: f64 = args[pos + 4].parse().unwrap_or(100.0);
            let rot: f64 = args[pos + 5].parse().unwrap_or(0.0);

            let shift = Transform2D::compute_shift([dx, dy], [sx, sy], rot);
            println!("{}", serde_json::json!({
                "delta_anchor": [dx, dy],
                "scale": [sx, sy],
                "rotation": rot,
                "position_shift": shift
            }));
            return;
        }
    }

    if let Some(pos) = args.iter().position(|a| a == "--solve-3d") {
        if args.len() >= pos + 10 {
            let dx: f64 = args[pos + 1].parse().unwrap_or(0.0);
            let dy: f64 = args[pos + 2].parse().unwrap_or(0.0);
            let dz: f64 = args[pos + 3].parse().unwrap_or(0.0);
            let sx: f64 = args[pos + 4].parse().unwrap_or(100.0);
            let sy: f64 = args[pos + 5].parse().unwrap_or(100.0);
            let sz: f64 = args[pos + 6].parse().unwrap_or(100.0);
            let rx: f64 = args[pos + 7].parse().unwrap_or(0.0);
            let ry: f64 = args[pos + 8].parse().unwrap_or(0.0);
            let rz: f64 = args[pos + 9].parse().unwrap_or(0.0);

            let shift = Transform3D::compute_shift([dx, dy, dz], [sx, sy, sz], [0.0; 3], [rx, ry, rz]);
            println!("{}", serde_json::json!({
                "delta_anchor": [dx, dy, dz],
                "scale": [sx, sy, sz],
                "rotation": [rx, ry, rz],
                "position_shift": shift
            }));
            return;
        }
    }

    eprintln!("Unknown command. Use --help for usage.");
}

fn run_benchmarks() {
    println!("=== PivotCraft Performance Benchmarks ===");

    // Benchmark 2D transform calculations
    let iters = 2_000_000;
    let t0 = Instant::now();
    let mut sum = 0.0;
    for i in 0..iters {
        let f = i as f64 * 0.001;
        let s = Transform2D::compute_shift([f, f * 0.5], [100.0 + f * 0.1, 100.0], f * 10.0);
        sum += s[0] + s[1];
    }
    let dur_2d = t0.elapsed();
    println!("2D Transform Shifts: {} ops in {:?} ({:.2} M ops/sec, sum={:.1})",
        iters, dur_2d, (iters as f64 / dur_2d.as_secs_f64()) / 1_000_000.0, sum);

    // Benchmark 3D transform calculations
    let iters_3d = 1_000_000;
    let t1 = Instant::now();
    let mut sum_3d = 0.0;
    for i in 0..iters_3d {
        let f = i as f64 * 0.001;
        let s = Transform3D::compute_shift([f, f * 0.5, f * 0.2], [100.0, 100.0, 100.0], [f, 0.0, 0.0], [0.0, f, f * 2.0]);
        sum_3d += s[0] + s[1] + s[2];
    }
    let dur_3d = t1.elapsed();
    println!("3D Transform Shifts: {} ops in {:?} ({:.2} M ops/sec, sum={:.1})",
        iters_3d, dur_3d, (iters_3d as f64 / dur_3d.as_secs_f64()) / 1_000_000.0, sum_3d);

    // Benchmark Alpha scanning
    let w = 1920;
    let h = 1080;
    let mut buf = vec![0.0f32; w * h * 4];
    // Fill middle 500x500 box with alpha 0.9
    for y in 290..790 {
        for x in 710..1210 {
            buf[(y * w + x) * 4 + 3] = 0.9;
        }
    }
    let t2 = Instant::now();
    let bounds = AlphaScanner::scan_rgba_f32(w, h, &buf, 0.05).unwrap();
    let dur_alpha = t2.elapsed();
    println!("Alpha Scanning (1080p frame): {:?} -> bounds=[{:.0}, {:.0}, {:.0}, {:.0}]",
        dur_alpha, bounds.x0, bounds.y0, bounds.x1, bounds.y1);
}
