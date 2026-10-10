use captioncraft::{parse_srt, parse_vtt, to_srt, to_vtt, check_overlaps, auto_fix_overlaps, shift_timing, sort_segments, CaptionSegment};
use std::env;
use std::fs;

fn load_subtitles(path: &str) -> Result<Vec<CaptionSegment>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read '{path}': {e}"))?;
    if path.ends_with(".vtt") || content.starts_with("WEBVTT") {
        parse_vtt(&content)
    } else {
        parse_srt(&content)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        println!("CaptionCraft CLI v0.1.0 - Subtitle & Caption Processor for EffectCraft");
        println!();
        println!("Usage:");
        println!("  captioncraft-cli --parse <file.srt|file.vtt>          Parse and display subtitle cues");
        println!("  captioncraft-cli --validate <file.srt|file.vtt>       Check for timing overlaps and issues");
        println!("  captioncraft-cli --convert <in_file> <out_file>       Convert between SRT and WebVTT");
        println!("  captioncraft-cli --fix-overlaps <in_file> <out_file>  Auto-fix overlapping caption cues");
        println!("  captioncraft-cli --shift <in_file> <out_file> <secs>  Shift subtitle timing by +/- seconds");
        return;
    }

    match args[1].as_str() {
        "--parse" => {
            let path = &args[2];
            match load_subtitles(path) {
                Ok(segs) => {
                    println!("Parsed {} caption cues from {}:", segs.len(), path);
                    println!("{:<4} {:<14} {:<14} {:<8} Text", "ID", "Start", "End", "Dur");
                    println!("{:-<64}", "");
                    for s in segs.iter().take(20) {
                        let preview: String = s.text.lines().next().unwrap_or("").chars().take(30).collect();
                        println!("{:<4} {:<14.3} {:<14.3} {:<8.2} {}", s.id, s.start_time, s.end_time, s.duration(), preview);
                    }
                    if segs.len() > 20 {
                        println!("... and {} more cues.", segs.len() - 20);
                    }
                }
                Err(e) => eprintln!("Error: {e}"),
            }
        }
        "--validate" => {
            let path = &args[2];
            match load_subtitles(path) {
                Ok(segs) => {
                    let overlaps = check_overlaps(&segs);
                    if overlaps.is_empty() {
                        println!("OK: No overlapping subtitles detected across {} cues.", segs.len());
                    } else {
                        eprintln!("Warning: Found {} overlapping subtitle pair(s):", overlaps.len());
                        for (a, b) in overlaps {
                            println!("  Cue #{} (ends {:.3}s) overlaps with Cue #{} (starts {:.3}s)",
                                segs[a].id, segs[a].end_time, segs[b].id, segs[b].start_time);
                        }
                    }
                }
                Err(e) => eprintln!("Error: {e}"),
            }
        }
        "--convert" => {
            let in_path = &args[2];
            let out_path = &args[3];
            let mut segs = load_subtitles(in_path).expect("load input");
            sort_segments(&mut segs);
            let output = if out_path.ends_with(".vtt") {
                to_vtt(&segs)
            } else {
                to_srt(&segs)
            };
            fs::write(out_path, output).expect("write output");
            println!("Converted {} cues from {} to {}", segs.len(), in_path, out_path);
        }
        "--fix-overlaps" => {
            let in_path = &args[2];
            let out_path = &args[3];
            let mut segs = load_subtitles(in_path).expect("load input");
            auto_fix_overlaps(&mut segs, 0.05);
            let output = if out_path.ends_with(".vtt") { to_vtt(&segs) } else { to_srt(&segs) };
            fs::write(out_path, output).expect("write output");
            println!("Fixed overlaps across {} cues and saved to {}", segs.len(), out_path);
        }
        "--shift" => {
            let in_path = &args[2];
            let out_path = &args[3];
            let delta: f64 = args[4].parse().expect("delta seconds");
            let mut segs = load_subtitles(in_path).expect("load input");
            shift_timing(&mut segs, delta);
            let output = if out_path.ends_with(".vtt") { to_vtt(&segs) } else { to_srt(&segs) };
            fs::write(out_path, output).expect("write output");
            println!("Shifted {} cues by {:.3}s and saved to {}", segs.len(), delta, out_path);
        }
        other => {
            eprintln!("Unknown argument: {other}. Use --help for usage.");
            std::process::exit(1);
        }
    }
}
