//! Comprehensive unit test suite for PasteCraft core crate.

use pastecraft::{AssetManager, Classifier, ClipboardKind, ReplacePreview, TargetLayer};

#[test]
fn test_empty_clipboard_rejection() {
    assert_eq!(Classifier::classify(""), ClipboardKind::Empty);
    assert_eq!(Classifier::classify("   "), ClipboardKind::Empty);
    assert_eq!(Classifier::classify("\n\t\r  \n"), ClipboardKind::Empty);
}

#[test]
fn test_invalid_unsupported_data() {
    let inputs = [
        "Lorem ipsum dolor sit amet",
        "some random unformatted string without tags or urls",
        "{ 'invalid_json': true }",
    ];
    for input in inputs {
        match Classifier::classify(input) {
            ClipboardKind::Unsupported { reason, .. } => {
                assert!(!reason.is_empty());
            }
            other => panic!("expected Unsupported for {input:?}, got {other:?}"),
        }
    }
}

#[test]
fn test_svg_vector_detection() {
    let valid_svgs = [
        r#"<svg viewBox="0 0 100 100"><rect width="100" height="100" fill="red"/></svg>"#,
        r#"<?xml version="1.0" encoding="UTF-8"?><svg width="200" height="200"><path d="M 0 0 L 100 100"/></svg>"#,
        r#"  <svg xmlns="http://www.w3.org/2000/svg"><g><circle r="10"/></g></svg>  "#,
    ];

    for svg in valid_svgs {
        match Classifier::classify(svg) {
            ClipboardKind::SvgVector { svg_xml } => {
                assert!(svg_xml.contains("<svg"));
            }
            other => panic!("expected SvgVector for {svg}, got {other:?}"),
        }
    }
}

#[test]
fn test_data_uri_png_decoding() {
    let png_b64 = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=";
    match Classifier::classify(png_b64) {
        ClipboardKind::ImageData { format, bytes_len } => {
            assert_eq!(format, "PNG");
            assert_eq!(bytes_len, 68);
        }
        other => panic!("expected ImageData, got {other:?}"),
    }
}

#[test]
fn test_data_uri_jpeg_decoding() {
    // Fake jpeg data uri with header \xFF\xD8\xFF
    let fake_jpeg_b64 = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEASABIAAD/";
    match Classifier::classify(fake_jpeg_b64) {
        ClipboardKind::ImageData { format, bytes_len } => {
            assert_eq!(format, "JPEG");
            assert!(bytes_len > 0);
        }
        other => panic!("expected ImageData, got {other:?}"),
    }
}

#[test]
fn test_web_url_classification() {
    let urls = [
        "https://cdn.example.com/images/hero.png",
        "http://assets.internal.net/vector_icon.svg",
        "https://unsplash.com/photos/xyz/download?force=true",
    ];

    for u in urls {
        match Classifier::classify(u) {
            ClipboardKind::WebUrl { url } => {
                assert_eq!(url, u);
            }
            other => panic!("expected WebUrl for {u}, got {other:?}"),
        }
    }
}

#[test]
fn test_file_paths_and_batch_lists() {
    let single = "/home/user/Pictures/frame_001.png";
    match Classifier::classify(single) {
        ClipboardKind::FilePaths { paths } => {
            assert_eq!(paths, vec![single]);
        }
        other => panic!("expected FilePaths, got {other:?}"),
    }

    let batch = "/media/clip1.mov\n/media/clip2.mov\n/media/clip3.mov";
    match Classifier::classify(batch) {
        ClipboardKind::FilePaths { paths } => {
            assert_eq!(paths.len(), 3);
            assert_eq!(paths[1], "/media/clip2.mov");
        }
        other => panic!("expected FilePaths for batch, got {other:?}"),
    }

    let file_uri = "file:///Users/artist/Desktop/badge.svg";
    match Classifier::classify(file_uri) {
        ClipboardKind::FilePaths { paths } => {
            assert_eq!(paths, vec!["/Users/artist/Desktop/badge.svg"]);
        }
        other => panic!("expected FilePaths for file URI, got {other:?}"),
    }
}

#[test]
fn test_destructive_replace_preview_generation() {
    let targets = vec![
        TargetLayer { id: 101, name: "Character Head".into(), current_source: "head_v1.png".into() },
        TargetLayer { id: 102, name: "Character Torso".into(), current_source: "torso_v1.png".into() },
        TargetLayer { id: 103, name: "Character Arm".into(), current_source: "arm_v1.png".into() },
    ];

    let preview = ReplacePreview::new(&targets, "Character_Rig_v2.svg");
    assert!(preview.requires_confirmation, "Must require explicit user confirmation");
    assert_eq!(preview.action, "replace");
    assert_eq!(preview.targets.len(), 3);
    assert_eq!(preview.targets[0].layer_name, "Character Head");
    assert!(preview.message.contains("3 layer(s)"));
    assert!(preview.message.contains("Character_Rig_v2.svg"));

    // Verify key preserved attributes are documented
    let props = preview.preserved_properties.join(" ");
    assert!(props.contains("transform"));
    assert!(props.contains("keyframes"));
    assert!(props.contains("effects"));
    assert!(props.contains("masks"));
    assert!(props.contains("timing"));
}

#[test]
fn test_repeated_operations_unique_naming() {
    let names: Vec<String> = (1..=5)
        .map(|i| AssetManager::format_asset_name("Pasted_Vector", i, "svg"))
        .collect();

    assert_eq!(names[0], "Pasted_Vector_1.svg");
    assert_eq!(names[1], "Pasted_Vector_2.svg");
    assert_eq!(names[2], "Pasted_Vector_3.svg");
    assert_eq!(names[3], "Pasted_Vector_4.svg");
    assert_eq!(names[4], "Pasted_Vector_5.svg");

    // All names must be unique
    let mut dedupped = names.clone();
    dedupped.sort();
    dedupped.dedup();
    assert_eq!(dedupped.len(), names.len());
}

#[test]
fn test_asset_name_sanitization() {
    assert_eq!(AssetManager::sanitize_prefix("Graphic 2026! (Draft)"), "Graphic_2026___Draft_");
    assert_eq!(AssetManager::sanitize_prefix(""), "Pasted_Asset");
    assert_eq!(AssetManager::sanitize_prefix("  ---  "), "__---__");
}
