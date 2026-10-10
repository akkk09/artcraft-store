//! PasteCraft CLI: standalone asset inspector, previewer, and batch pipeline utility.

use std::env;
use pastecraft::{Classifier, ClipboardKind, ReplacePreview, TargetLayer};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("PasteCraft CLI v0.1.0 - Universal Clipboard Asset Utility for EffectCraft");
        println!("Usage:");
        println!("  pastecraft-cli inspect <data_string>");
        println!("  pastecraft-cli preview <incoming_name> <layer_name_1> [layer_name_2...]");
        println!("  pastecraft-cli decode-check <base64_or_data_uri>");
        return;
    }

    match args[1].as_str() {
        "inspect" => {
            let data = args.get(2).map(|s| s.as_str()).unwrap_or("");
            let kind = Classifier::classify(data);
            match &kind {
                ClipboardKind::Empty => println!("Status: EMPTY"),
                ClipboardKind::ImageData { format, bytes_len } => println!("Status: IMAGE | Format: {format} | Size: {bytes_len} bytes"),
                ClipboardKind::SvgVector { svg_xml } => println!("Status: SVG | Size: {} bytes", svg_xml.len()),
                ClipboardKind::WebUrl { url } => println!("Status: URL | Target: {url}"),
                ClipboardKind::FilePaths { paths } => println!("Status: FILES | Count: {}", paths.len()),
                ClipboardKind::Unsupported { reason, .. } => println!("Status: UNSUPPORTED | Reason: {reason}"),
            }
            if let Ok(json) = serde_json::to_string_pretty(&kind) {
                println!("{json}");
            }
        }
        "preview" => {
            let incoming = args.get(2).map(|s| s.as_str()).unwrap_or("Pasted_Asset.png");
            let layer_names: Vec<&str> = if args.len() > 3 {
                args[3..].iter().map(|s| s.as_str()).collect()
            } else {
                vec!["Selected Layer 1"]
            };

            let targets: Vec<TargetLayer> = layer_names
                .iter()
                .enumerate()
                .map(|(i, name)| TargetLayer {
                    id: (i + 1) as u64,
                    name: name.to_string(),
                    current_source: format!("Source_{name}"),
                })
                .collect();

            let preview = ReplacePreview::new(&targets, incoming);
            println!("{}", serde_json::to_string_pretty(&preview).unwrap());
        }
        "decode-check" => {
            let input = args.get(2).map(|s| s.as_str()).unwrap_or("");
            let kind = Classifier::classify(input);
            match kind {
                ClipboardKind::ImageData { format, bytes_len } => {
                    println!("Successfully decoded {format} image of {bytes_len} bytes");
                }
                _ => {
                    println!("Input is not a recognized base64 or data-uri image");
                }
            }
        }
        other => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}
