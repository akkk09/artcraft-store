//! QuietCraft CLI: Standalone silence detection and timeline cut generator.

use std::env;
use std::fs;
use std::process;

use quietcraft::audio::parse_wav;
use quietcraft::{analyze_energy_windows, detect_silence, export_csv, export_edl, export_json, QuietParams, built_in_presets};

fn print_usage() {
    eprintln!(
        "QuietCraft CLI: Automated silence detection and ripple cut generator\n\
         Usage:\n\
           quietcraft-cli analyze <file.wav> [threshold_db] [min_silence_sec] [pre_roll] [post_roll]\n\
           quietcraft-cli cuts <file.wav> [threshold_db] [-o cuts.json]\n\
           quietcraft-cli edl <file.wav> [threshold_db] [-o cuts.edl]\n\
           quietcraft-cli csv <file.wav> [threshold_db]\n\
           quietcraft-cli presets\n"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "presets" => {
            println!("Built-in QuietCraft Presets:");
            for p in built_in_presets() {
                println!(
                    " - {}: {} (Threshold: {:.1} dB, Min Silence: {:.2}s, Pre: {:.2}s, Post: {:.2}s)",
                    p.name, p.description, p.params.threshold_db, p.params.min_silence_dur, p.params.pre_roll_pad, p.params.post_roll_pad
                );
            }
        }
        "analyze" | "cuts" | "edl" | "csv" => {
            if args.len() < 3 {
                eprintln!("Error: missing audio file path");
                print_usage();
                process::exit(1);
            }
            let file_path = &args[2];
            let data = match fs::read(file_path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error reading {file_path}: {e}");
                    process::exit(1);
                }
            };
            let buf = match parse_wav(&data) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Error parsing WAV {file_path}: {e}");
                    process::exit(1);
                }
            };

            let mut params = QuietParams::default();
            if args.len() > 3 {
                if let Ok(v) = args[3].parse::<f64>() {
                    params.threshold_db = v;
                }
            }
            if args.len() > 4 {
                if let Ok(v) = args[4].parse::<f64>() {
                    params.min_silence_dur = v;
                }
            }
            if args.len() > 5 {
                if let Ok(v) = args[5].parse::<f64>() {
                    params.pre_roll_pad = v;
                }
            }
            if args.len() > 6 {
                if let Ok(v) = args[6].parse::<f64>() {
                    params.post_roll_pad = v;
                }
            }

            let windows = analyze_energy_windows(&buf.samples, buf.sample_rate, 0.025, 0.010);
            let report = detect_silence(&windows, buf.duration, &params);

            match args[1].as_str() {
                "analyze" => {
                    println!("--- QuietCraft Analysis Report ---");
                    println!("File: {file_path}");
                    println!("Sample Rate: {} Hz, Channels: {}", buf.sample_rate, buf.channels);
                    println!("Original Duration: {:.2}s", report.total_duration);
                    println!("Tightened Duration: {:.2}s", report.tightened_duration);
                    println!("Silence Removed: {:.2}s ({:.1}% time saved)", report.silence_removed, report.time_saved_pct);
                    println!("Total Cuts: {}", report.cut_count);
                    println!("Speech Segments: {}", report.speech_segments.len());
                    for s in &report.speech_segments {
                        println!("  #{} Speech: {:.3}s -> {:.3}s (dur: {:.3}s)", s.id, s.start, s.end, s.duration());
                    }
                    println!("Silence Gaps: {}", report.silence_segments.len());
                    for s in &report.silence_segments {
                        println!("  #{} Silence: {:.3}s -> {:.3}s (dur: {:.3}s)", s.id, s.start, s.end, s.duration());
                    }
                }
                "cuts" => {
                    println!("{}", export_json(&report));
                }
                "edl" => {
                    println!("{}", export_edl(&report, file_path));
                }
                "csv" => {
                    print!("{}", export_csv(&report));
                }
                _ => unreachable!(),
            }
        }
        other => {
            eprintln!("Unknown command: {other}");
            print_usage();
            process::exit(1);
        }
    }
}
