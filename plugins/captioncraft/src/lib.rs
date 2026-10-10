//! CaptionCraft Core: subtitle parser, segment timing editor, typography styler,
//! and format exporter for EffectCraft.

use serde::{Deserialize, Serialize};

/// Represents a single subtitle or caption cue.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaptionSegment {
    pub id: usize,
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
}

impl CaptionSegment {
    pub fn new(id: usize, start: f64, end: f64, text: impl Into<String>) -> Result<Self, String> {
        if !start.is_finite() || !end.is_finite() {
            return Err("Time values must be finite numbers".to_string());
        }
        let start_time = start.max(0.0);
        let end_time = end.max(start_time);
        Ok(Self { id, start_time, end_time, text: text.into() })
    }

    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }

    pub fn is_active_at(&self, time: f64) -> bool {
        time >= self.start_time && time < self.end_time
    }
}

/// Typography and layout styling for rendered caption layers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaptionStyle {
    pub font_family: String,
    pub font_size: f64,
    pub fill_color: [f32; 4],
    pub stroke_color: [f32; 4],
    pub stroke_width: f64,
    pub justification: String,
    pub margin_bottom_pct: f64,
}

impl Default for CaptionStyle {
    fn default() -> Self {
        Self {
            font_family: "Inter".into(),
            font_size: 48.0,
            fill_color: [1.0, 1.0, 1.0, 1.0], // High contrast white
            stroke_color: [0.0, 0.0, 0.0, 1.0], // Black outline
            stroke_width: 3.5,
            justification: "center".into(),
            margin_bottom_pct: 0.12, // Lower-third safe margin (12% from bottom)
        }
    }
}

/// A complete caption track containing cues and styling settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaptionTrack {
    pub title: String,
    pub style: CaptionStyle,
    pub segments: Vec<CaptionSegment>,
}

impl Default for CaptionTrack {
    fn default() -> Self {
        Self {
            title: "Captions".into(),
            style: CaptionStyle::default(),
            segments: Vec::new(),
        }
    }
}

// --- Timecode Parsing and Formatting ---

/// Parses timecodes from either SRT (`00:01:23,456`) or WebVTT (`00:01:23.456` or `01:23.456`).
pub fn parse_timecode(s: &str) -> Result<f64, String> {
    let clean = s.trim().replace(',', ".");
    let parts: Vec<&str> = clean.split(':').collect();
    match parts.len() {
        3 => {
            let h: f64 = parts[0].parse().map_err(|e| format!("Invalid hours: {e}"))?;
            let m: f64 = parts[1].parse().map_err(|e| format!("Invalid minutes: {e}"))?;
            let s: f64 = parts[2].parse().map_err(|e| format!("Invalid seconds: {e}"))?;
            Ok(h * 3600.0 + m * 60.0 + s)
        }
        2 => {
            let m: f64 = parts[0].parse().map_err(|e| format!("Invalid minutes: {e}"))?;
            let s: f64 = parts[1].parse().map_err(|e| format!("Invalid seconds: {e}"))?;
            Ok(m * 60.0 + s)
        }
        1 => {
            let s: f64 = parts[0].parse().map_err(|e| format!("Invalid seconds: {e}"))?;
            Ok(s)
        }
        _ => Err(format!("Unrecognized timecode format: '{s}'")),
    }
}

/// Formats seconds as SRT timecode: `hh:mm:ss,mmm`.
pub fn format_timecode_srt(secs: f64) -> String {
    let secs = secs.max(0.0);
    let total_ms = (secs * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total_s = total_ms / 1000;
    let s = total_s % 60;
    let m = (total_s / 60) % 60;
    let h = total_s / 3600;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
}

/// Formats seconds as WebVTT timecode: `hh:mm:ss.mmm`.
pub fn format_timecode_vtt(secs: f64) -> String {
    let secs = secs.max(0.0);
    let total_ms = (secs * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total_s = total_ms / 1000;
    let s = total_s % 60;
    let m = (total_s / 60) % 60;
    let h = total_s / 3600;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
}

// --- SRT & WebVTT Parsers and Serializers ---

/// Parses SubRip (.srt) text content into caption segments.
pub fn parse_srt(input: &str) -> Result<Vec<CaptionSegment>, String> {
    let mut segments = Vec::new();
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let blocks: Vec<&str> = normalized.split("\n\n").collect();

    for block in blocks {
        let lines: Vec<&str> = block.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        if lines.is_empty() {
            continue;
        }

        let (idx_line, timing_line, text_lines) = if lines.len() >= 2 && lines[1].contains("-->") {
            (lines[0], lines[1], &lines[2..])
        } else if lines[0].contains("-->") {
            ("0", lines[0], &lines[1..])
        } else {
            continue;
        };

        let id: usize = idx_line.parse().unwrap_or(segments.len() + 1);
        let arrow_parts: Vec<&str> = timing_line.split("-->").collect();
        if arrow_parts.len() != 2 {
            continue;
        }

        let start = parse_timecode(arrow_parts[0])?;
        let end = parse_timecode(arrow_parts[1])?;
        let text = text_lines.join("\n");

        if !text.is_empty() {
            segments.push(CaptionSegment::new(id, start, end, text)?);
        }
    }

    Ok(segments)
}

/// Parses WebVTT (.vtt) text content into caption segments.
pub fn parse_vtt(input: &str) -> Result<Vec<CaptionSegment>, String> {
    let mut segments = Vec::new();
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");

    let content = if let Some(stripped) = normalized.strip_prefix("WEBVTT") {
        stripped
    } else {
        &normalized
    };

    let blocks: Vec<&str> = content.split("\n\n").collect();

    for block in blocks {
        let lines: Vec<&str> = block.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        if lines.is_empty() {
            continue;
        }

        // Handle optional cue identifier line
        let (timing_line, text_lines) = if lines[0].contains("-->") {
            (lines[0], &lines[1..])
        } else if lines.len() >= 2 && lines[1].contains("-->") {
            (lines[1], &lines[2..])
        } else {
            continue;
        };

        // WebVTT timing line can include cue settings after the timestamp (e.g. `align:center position:50%`)
        let arrow_parts: Vec<&str> = timing_line.split("-->").collect();
        if arrow_parts.len() != 2 {
            continue;
        }

        let start = parse_timecode(arrow_parts[0].trim())?;
        let end_token = arrow_parts[1].trim().split_whitespace().next().unwrap_or("");
        let end = parse_timecode(end_token)?;

        let text = text_lines.join("\n");
        if !text.is_empty() {
            segments.push(CaptionSegment::new(segments.len() + 1, start, end, text)?);
        }
    }

    Ok(segments)
}

/// Serializes caption segments into clean SubRip (.srt) formatted text.
pub fn to_srt(segments: &[CaptionSegment]) -> String {
    let mut out = String::new();
    for (i, seg) in segments.iter().enumerate() {
        out.push_str(&format!("{}\n", i + 1));
        out.push_str(&format!("{} --> {}\n", format_timecode_srt(seg.start_time), format_timecode_srt(seg.end_time)));
        out.push_str(&seg.text);
        out.push_str("\n\n");
    }
    out
}

/// Serializes caption segments into clean WebVTT (.vtt) formatted text.
pub fn to_vtt(segments: &[CaptionSegment]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for (i, seg) in segments.iter().enumerate() {
        out.push_str(&format!("{}\n", i + 1));
        out.push_str(&format!("{} --> {}\n", format_timecode_vtt(seg.start_time), format_timecode_vtt(seg.end_time)));
        out.push_str(&seg.text);
        out.push_str("\n\n");
    }
    out
}

// --- Segment Operations & Conflict Auditing ---

/// Scans for overlapping caption intervals. Returns index pairs `(i, i+1)` where `start[i+1] < end[i]`.
pub fn check_overlaps(segments: &[CaptionSegment]) -> Vec<(usize, usize)> {
    let mut overlaps = Vec::new();
    for i in 0..segments.len().saturating_sub(1) {
        if segments[i + 1].start_time < segments[i].end_time {
            overlaps.push((i, i + 1));
        }
    }
    overlaps
}

/// Resolves overlapping segments by trimming each preceding cue's end time to `start[i+1] - gap`.
pub fn auto_fix_overlaps(segments: &mut [CaptionSegment], gap: f64) {
    for i in 0..segments.len().saturating_sub(1) {
        if segments[i + 1].start_time < segments[i].end_time {
            let new_end = (segments[i + 1].start_time - gap).max(segments[i].start_time + 0.1);
            segments[i].end_time = new_end;
        }
    }
}

/// Shifts all segments by a delta in seconds, preventing negative start times.
pub fn shift_timing(segments: &mut [CaptionSegment], delta: f64) {
    for seg in segments.iter_mut() {
        let dur = seg.duration();
        seg.start_time = (seg.start_time + delta).max(0.0);
        seg.end_time = seg.start_time + dur;
    }
}

/// Splits a segment into two at `split_time`.
pub fn split_segment(segments: &mut Vec<CaptionSegment>, idx: usize, split_time: f64) -> Result<(), String> {
    if idx >= segments.len() {
        return Err("Segment index out of range".to_string());
    }
    let seg = &segments[idx];
    if split_time <= seg.start_time || split_time >= seg.end_time {
        return Err("Split time must be strictly within the segment duration".to_string());
    }

    let orig_end = seg.end_time;
    let full_text = seg.text.clone();

    // Divide text roughly at words or halfway
    let words: Vec<&str> = full_text.split_whitespace().collect();
    let (t1, t2) = if words.len() >= 2 {
        let mid = words.len() / 2;
        (words[..mid].join(" "), words[mid..].join(" "))
    } else {
        (full_text.clone(), full_text)
    };

    segments[idx].end_time = split_time;
    segments[idx].text = t1;

    let new_seg = CaptionSegment {
        id: segments.len() + 1,
        start_time: split_time,
        end_time: orig_end,
        text: t2,
    };
    segments.insert(idx + 1, new_seg);
    reindex_segments(segments);
    Ok(())
}

/// Merges segment `idx` with `idx + 1`.
pub fn merge_segments(segments: &mut Vec<CaptionSegment>, idx: usize) -> Result<(), String> {
    if idx + 1 >= segments.len() {
        return Err("Cannot merge last segment (requires next segment)".to_string());
    }
    let next = segments.remove(idx + 1);
    segments[idx].end_time = next.end_time;
    segments[idx].text = format!("{}\n{}", segments[idx].text.trim(), next.text.trim());
    reindex_segments(segments);
    Ok(())
}

/// Sorts segments chronologically by start time and re-numbers IDs.
pub fn sort_segments(segments: &mut [CaptionSegment]) {
    segments.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap_or(std::cmp::Ordering::Equal));
    reindex_segments(segments);
}

fn reindex_segments(segments: &mut [CaptionSegment]) {
    for (i, seg) in segments.iter_mut().enumerate() {
        seg.id = i + 1;
    }
}
