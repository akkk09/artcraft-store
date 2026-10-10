//! depthcraft-cli: Monocular depth estimation batch and video preprocessor for EffectCraft.

use std::time::Instant;

use depthcraft::models::{MODELS, QualityPreset};
use depthcraft::pipeline::{CancellationToken, DepthPipeline, PipelineConfig};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--models" || a == "models") {
        println!("DepthCraft Supported Depth Models:");
        println!("----------------------------------");
        for m in MODELS {
            println!("ID:          {}", m.id);
            println!("Name:        {}", m.name);
            println!("License:     {}", m.license);
            println!("License URL: {}", m.license_url);
            println!("Homepage:    {}", m.homepage);
            println!("Default Res: {}x{}", m.default_resolution.0, m.default_resolution.1);
            println!("Weights:     {}", m.weights_file);
            println!("SHA-256:     {}", m.sha256);
            println!("Description: {}", m.description);
            println!();
        }
        return;
    }

    if args.iter().any(|a| a == "--bench" || a == "bench") {
        run_benchmark();
        return;
    }

    println!("DepthCraft CLI - Monocular Depth Estimation for EffectCraft");
    println!("Usage:");
    println!("  depthcraft-cli --models       List supported depth estimation models & licenses");
    println!("  depthcraft-cli --bench        Run CPU depth estimation benchmark on synthetic video frames");
}

fn run_benchmark() {
    println!("Running DepthCraft CPU Depth Estimation Benchmark...");
    println!("--------------------------------------------------");

    let width = 640;
    let height = 360;
    let frame_count = 10;

    println!("Generating {frame_count} test video frames ({width}x{height})...");
    let mut frames_data: Vec<Vec<[f32; 3]>> = Vec::with_capacity(frame_count);

    for f in 0..frame_count {
        let mut frame = vec![[0.0f32; 3]; width * height];
        let t = f as f32 * 0.1;

        for y in 0..height {
            for x in 0..width {
                // Synthetic landscape with foreground moving sphere and background gradient
                let nx = x as f32 / width as f32;
                let ny = y as f32 / height as f32;

                let cx = 0.5 + 0.2 * (t * 2.0).sin();
                let cy = 0.6;
                let dist = ((nx - cx).powi(2) + (ny - cy).powi(2)).sqrt();

                let is_sphere = dist < 0.15;
                let col = if is_sphere {
                    [0.9, 0.2, 0.2] // Red foreground ball
                } else {
                    [0.2 + 0.6 * ny, 0.4 + 0.4 * ny, 0.8] // Background sky/ground gradient
                };

                frame[y * width + x] = col;
            }
        }
        frames_data.push(frame);
    }

    let frames_refs: Vec<(&[[f32; 3]], usize, usize)> = frames_data.iter().map(|f| (f.as_slice(), width, height)).collect();

    let config = PipelineConfig {
        model_id: "classical".to_string(),
        quality: QualityPreset::Balanced,
        edge_refine_radius: 2,
        temporal_smooth_weight: 50.0,
        memory_limit_bytes: 64 * 1024 * 1024,
    };

    let mut pipeline = DepthPipeline::new(config);
    let cancel = CancellationToken::new();

    let start = Instant::now();
    let callback = Box::new(|report: depthcraft::pipeline::ProgressReport| {
        println!(
            "  Frame {}/{} ({:.1}%) - Cache hit: {} - Memory: {:.2} MB",
            report.current_frame,
            report.total_frames,
            report.percentage,
            report.cache_hit,
            report.memory_used_bytes as f64 / (1024.0 * 1024.0)
        );
    });

    let results = pipeline.process_sequence(&frames_refs, Some(callback), &cancel);
    let elapsed = start.elapsed();

    match results {
        Ok(depths) => {
            let total_ms = elapsed.as_secs_f64() * 1000.0;
            let per_frame_ms = total_ms / frame_count as f64;
            let fps = frame_count as f64 / elapsed.as_secs_f64();

            println!("Benchmark complete:");
            println!("  Frames processed: {frame_count}");
            println!("  Total time:       {total_ms:.2} ms");
            println!("  Per frame:        {per_frame_ms:.2} ms");
            println!("  Throughput:       {fps:.2} fps");
            println!("  Final memory:     {:.2} MB", pipeline.cache_memory_bytes() as f64 / (1024.0 * 1024.0));
            assert_eq!(depths.len(), frame_count);
        }
        Err(e) => {
            eprintln!("Benchmark failed: {e}");
        }
    }
}
