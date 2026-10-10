//! PasteCraft: Universal clipboard asset importer and layer replacer for EffectCraft.
//!
//! Provides clipboard classification, base64 data decoding, asset naming,
//! destructive change previewing, and transform preservation contracts.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Standard RFC 4648 Base64 decoder with whitespace and padding tolerance.
pub fn decode_base64(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u32;
    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\r' | b'\n' | b' ' | b'\t' => continue,
            _ => return None,
        };
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(out)
}

/// Detect image extension from magic bytes.
pub fn detect_image_ext(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        "png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "gif"
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "webp"
    } else {
        "png"
    }
}

/// The classified data kind from clipboard or user input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClipboardKind {
    Empty,
    ImageData { format: String, bytes_len: usize },
    SvgVector { svg_xml: String },
    WebUrl { url: String },
    FilePaths { paths: Vec<String> },
    Unsupported { text: String, reason: String },
}

/// Check if a string is likely a file path.
pub fn is_likely_file_path(p_str: &str) -> bool {
    let p = Path::new(p_str);
    if p.is_file() {
        return true;
    }
    if p_str.starts_with('/') || p_str.starts_with("./") || p_str.starts_with("../") || p_str.starts_with("file://") {
        return true;
    }
    if !p_str.contains('\n') && !p_str.contains('\r') {
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            let lower = ext.to_ascii_lowercase();
            const MEDIA_EXTS: &[&str] = &[
                "png", "jpg", "jpeg", "gif", "webp", "svg", "mp4", "mov", "avi", "mkv",
                "wav", "mp3", "aac", "psd", "psb", "ai", "pdf", "eps", "tif", "tiff", "exr", "wat", "wasm", "json", "csv"
            ];
            if MEDIA_EXTS.contains(&lower.as_str()) && !p_str.contains(' ') {
                return true;
            }
        }
    }
    false
}

/// Classifier for raw clipboard / text input.
pub struct Classifier;

impl Classifier {
    pub fn classify(raw: &str) -> ClipboardKind {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return ClipboardKind::Empty;
        }

        // 1. Data URI image
        if trimmed.starts_with("data:image/") {
            if let Some(pos) = trimmed.find(";base64,") {
                let format = &trimmed[11..pos];
                let b64 = &trimmed[pos + 8..];
                if let Some(bytes) = decode_base64(b64) {
                    return ClipboardKind::ImageData {
                        format: format.to_uppercase(),
                        bytes_len: bytes.len(),
                    };
                }
            }
        }

        // 2. SVG Vector markup
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("<svg") || (lower.starts_with("<?xml") && lower.contains("<svg")) {
            return ClipboardKind::SvgVector {
                svg_xml: trimmed.to_string(),
            };
        }

        // 3. Web URL
        if lower.starts_with("http://") || lower.starts_with("https://") {
            return ClipboardKind::WebUrl {
                url: trimmed.to_string(),
            };
        }

        // 4. File URI(s) or Local File Path(s)
        let lines: Vec<&str> = trimmed.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        let mut resolved_paths = Vec::new();
        let mut all_files = true;

        for line in &lines {
            let p_str = if line.starts_with("file://") {
                &line[7..]
            } else {
                *line
            };
            if is_likely_file_path(p_str) {
                resolved_paths.push(p_str.to_string());
            } else {
                all_files = false;
                break;
            }
        }

        if all_files && !resolved_paths.is_empty() {
            return ClipboardKind::FilePaths {
                paths: resolved_paths,
            };
        }

        // 5. Raw Base64 encoded image
        if trimmed.len() > 64 && !trimmed.contains('\n') {
            if let Some(bytes) = decode_base64(trimmed) {
                let ext = detect_image_ext(&bytes);
                if ext == "png" || ext == "jpg" || ext == "gif" || ext == "webp" {
                    return ClipboardKind::ImageData {
                        format: ext.to_uppercase(),
                        bytes_len: bytes.len(),
                    };
                }
            }
        }

        ClipboardKind::Unsupported {
            text: trimmed.chars().take(80).collect(),
            reason: "Expected image data, SVG vector code, web URL, or local file path".into(),
        }
    }
}

/// Asset naming and file path manager.
pub struct AssetManager;

impl AssetManager {
    /// Sanitize a string into a safe filename prefix.
    pub fn sanitize_prefix(prefix: &str) -> String {
        let clean: String = prefix
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect();
        if clean.is_empty() {
            "Pasted_Asset".to_string()
        } else {
            clean
        }
    }

    /// Generate an incremented asset filename.
    pub fn format_asset_name(prefix: &str, index: usize, ext: &str) -> String {
        let clean = Self::sanitize_prefix(prefix);
        format!("{clean}_{index}.{ext}")
    }

    /// Resolves the default asset folder relative to the project.
    pub fn default_asset_dir(project_path: Option<&str>) -> PathBuf {
        if let Some(p) = project_path {
            if let Some(parent) = Path::new(p).parent() {
                return parent.join("(Assets)");
            }
        }
        PathBuf::from("/tmp/effectcraft_pastecraft")
    }
}

/// Representation of a target layer to be replaced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetLayer {
    pub id: u64,
    pub name: String,
    pub current_source: String,
}

/// Preview description for a destructive replace operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacePreview {
    pub requires_confirmation: bool,
    pub action: String,
    pub message: String,
    pub targets: Vec<TargetLayerPreview>,
    pub preserved_properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetLayerPreview {
    pub layer_id: u64,
    pub layer_name: String,
    pub current_source: String,
}

impl ReplacePreview {
    pub fn new(targets: &[TargetLayer], incoming_desc: &str) -> Self {
        let preserved = vec![
            "transform (position, scale, rotation, opacity, anchorPoint)".to_string(),
            "animated keyframes & expressions".to_string(),
            "effects stack".to_string(),
            "masks and matte options".to_string(),
            "timing (inPoint, outPoint, startTime, stretch)".to_string(),
            "parent and track matte hierarchy".to_string(),
        ];

        let target_previews: Vec<TargetLayerPreview> = targets
            .iter()
            .map(|t| TargetLayerPreview {
                layer_id: t.id,
                layer_name: t.name.clone(),
                current_source: t.current_source.clone(),
            })
            .collect();

        Self {
            requires_confirmation: true,
            action: "replace".to_string(),
            message: format!(
                "Destructive change: Replace source of {} layer(s) with {incoming_desc}. Preserves all keyframes and effects.",
                targets.len()
            ),
            targets: target_previews,
            preserved_properties: preserved,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_empty_string() {
        assert_eq!(Classifier::classify(""), ClipboardKind::Empty);
        assert_eq!(Classifier::classify("   \n\t  "), ClipboardKind::Empty);
    }

    #[test]
    fn test_classify_svg_markup() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><circle cx="50" cy="50" r="40"/></svg>"#;
        match Classifier::classify(svg) {
            ClipboardKind::SvgVector { svg_xml } => {
                assert!(svg_xml.contains("circle"));
            }
            other => panic!("expected SvgVector, got {other:?}"),
        }
    }

    #[test]
    fn test_classify_data_uri_png() {
        // Minimal 1x1 transparent PNG encoded in base64
        let data_uri = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=";
        match Classifier::classify(data_uri) {
            ClipboardKind::ImageData { format, bytes_len } => {
                assert_eq!(format, "PNG");
                assert!(bytes_len > 10);
            }
            other => panic!("expected ImageData, got {other:?}"),
        }
    }

    #[test]
    fn test_classify_web_urls() {
        let url = "https://example.com/assets/graphic.png";
        assert_eq!(Classifier::classify(url), ClipboardKind::WebUrl { url: url.to_string() });
    }

    #[test]
    fn test_classify_file_paths() {
        let paths = "/tmp/image1.png\n/tmp/image2.jpg";
        match Classifier::classify(paths) {
            ClipboardKind::FilePaths { paths: p } => {
                assert_eq!(p.len(), 2);
                assert_eq!(p[0], "/tmp/image1.png");
            }
            other => panic!("expected FilePaths, got {other:?}"),
        }
    }

    #[test]
    fn test_classify_unsupported_text() {
        let random = "This is just some plain sentences that are neither URLs nor SVGs.";
        match Classifier::classify(random) {
            ClipboardKind::Unsupported { .. } => {}
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }

    #[test]
    fn test_replace_preview_contract() {
        let targets = vec![
            TargetLayer { id: 1, name: "Logo".into(), current_source: "old_logo.png".into() },
            TargetLayer { id: 2, name: "Background".into(), current_source: "bg.mp4".into() },
        ];
        let preview = ReplacePreview::new(&targets, "new_logo.png");
        assert!(preview.requires_confirmation);
        assert_eq!(preview.targets.len(), 2);
        assert!(preview.preserved_properties.len() >= 5);
        assert!(preview.message.contains("new_logo.png"));
    }

    #[test]
    fn test_asset_manager_sanitization() {
        assert_eq!(AssetManager::sanitize_prefix("My Cool Graphic!"), "My_Cool_Graphic_");
        assert_eq!(AssetManager::format_asset_name("Graphic", 3, "png"), "Graphic_3.png");
    }
}
