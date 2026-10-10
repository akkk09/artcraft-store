//! Citron CLI benchmark and curve testing utility.

use std::time::Instant;
use citron::{
    BufferSnapshot, ChannelCurve, KeyPoint, LatticeAnchor, LatticeCage, PresetTangents,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("bench");

    match mode {
        "info" => {
            println!("Citron: Professional Graph Editor & Curve Engine for EffectCraft");
            println!("Version: 1.0.0");
            println!("Features: Multi-channel visualization, FFD lattice cage, de Casteljau split, buffer comparison.");
        }
        "bench" | _ => {
            println!("Running Citron Engine Performance Benchmark...");

            // Create a channel with 10,000 keyframes
            let mut ch = ChannelCurve::new("pos_x", "Position X", [1.0, 0.3, 0.3, 1.0]);
            let num_keys = 10_000;
            for i in 0..num_keys {
                let t = i as f64 * 0.1;
                let v = (t * 0.5).sin() * 500.0;
                let (ease_in, ease_out) = PresetTangents::strong_punch();
                let mut k = KeyPoint::new(t, v);
                k.in_tangent = ease_in;
                k.out_tangent = ease_out;
                ch.keys.push(k);
            }

            // Benchmark 1: Bézier evaluations
            let start = Instant::now();
            let mut sum = 0.0;
            for i in 0..50_000 {
                let t = (i as f64) * 0.02;
                sum += ch.eval(t);
            }
            let eval_elapsed = start.elapsed();
            println!("50,000 Bézier curve evaluations completed in {:.2?} (sum={:.2})", eval_elapsed, sum);

            // Benchmark 2: Lattice Cage FFD transformation
            let cage = LatticeCage {
                time_scale: 1.25,
                value_scale: 0.8,
                time_offset: 0.5,
                value_offset: 100.0,
                time_skew: 0.15,
                anchor: LatticeAnchor::Center,
            };

            let start_cage = Instant::now();
            cage.transform_keys(&mut ch.keys);
            let cage_elapsed = start_cage.elapsed();
            println!("Lattice FFD cage transformed {} keys in {:.2?}", num_keys, cage_elapsed);

            // Benchmark 3: Buffer comparison
            let snapshot = BufferSnapshot::capture(&[ch.clone()]);
            let start_rmse = Instant::now();
            let rmse = snapshot.compute_delta_rmse(&[ch], 1_000);
            let rmse_elapsed = start_rmse.elapsed();
            println!("Buffer RMSE comparison completed in {:.2?} (delta={:.4})", rmse_elapsed, rmse);
        }
    }
}
