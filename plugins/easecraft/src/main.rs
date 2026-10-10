use easecraft::{EaseCurve, PresetPack};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        println!("EaseCraft CLI v0.1.0 - Keyframe Curve Solver & Tool for EffectCraft");
        println!();
        println!("Usage:");
        println!("  easecraft-cli --presets                 List all standard and dynamic presets");
        println!("  easecraft-cli --export <file.json>      Export built-in presets to open JSON format");
        println!("  easecraft-cli --eval <outInf> <outSpd> <inInf> <inSpd> [steps]");
        println!("                                          Evaluate and print curve progression");
        println!("  easecraft-cli --bench                   Run solver accuracy and performance benchmark");
        return;
    }

    match args[1].as_str() {
        "--presets" => {
            let pack = PresetPack::default();
            println!("{:<24} {:<12} {:<10} {:<10} {:<10} {:<10}", "Name", "Category", "OutInf%", "OutSpd", "InInf%", "InSpd");
            println!("{:-<76}", "");
            for p in &pack.presets {
                println!(
                    "{:<24} {:<12} {:<10.2} {:<10.2} {:<10.2} {:<10.2}",
                    p.name, p.category, p.curve.out_influence, p.curve.out_speed, p.curve.in_influence, p.curve.in_speed
                );
            }
        }
        "--export" => {
            let path = args.get(2).cloned().unwrap_or_else(|| "easecraft_presets.json".to_string());
            let pack = PresetPack::default();
            let json = pack.to_json().expect("serialize presets");
            std::fs::write(&path, json).expect("write file");
            println!("Exported {} presets to {}", pack.presets.len(), path);
        }
        "--eval" => {
            if args.len() < 6 {
                eprintln!("Error: --eval requires <outInf> <outSpd> <inInf> <inSpd>");
                std::process::exit(1);
            }
            let out_inf: f64 = args[2].parse().expect("outInf");
            let out_spd: f64 = args[3].parse().expect("outSpd");
            let in_inf: f64 = args[4].parse().expect("inInf");
            let in_spd: f64 = args[5].parse().expect("inSpd");
            let steps: usize = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(10);

            let curve = EaseCurve::new(out_inf, out_spd, in_inf, in_spd).expect("valid curve");
            println!("Evaluating curve [Out: {:.1}% @ {:.2}, In: {:.1}% @ {:.2}] across {} steps:", curve.out_influence, curve.out_speed, curve.in_influence, curve.in_speed, steps);
            for s in 0..=steps {
                let x = s as f64 / steps as f64;
                let y = curve.eval(x);
                println!("  t = {:.3} -> progress = {:.4}", x, y);
            }
        }
        "--bench" => {
            println!("Benchmarking EaseCraft Newton-Raphson curve solver...");
            let curve = EaseCurve::back_in_out();
            let start = std::time::Instant::now();
            let iters = 1_000_000;
            let mut sum = 0.0;
            for i in 0..iters {
                let x = (i % 1000) as f64 / 1000.0;
                sum += curve.eval(x);
            }
            let dur = start.elapsed();
            println!("Evaluated {} curve points in {:?}", iters, dur);
            println!("Throughput: {:.2} million points/second (checksum = {:.2})", (iters as f64 / dur.as_secs_f64()) / 1e6, sum);
        }
        other => {
            eprintln!("Unknown argument: {other}. Use --help for usage.");
            std::process::exit(1);
        }
    }
}
