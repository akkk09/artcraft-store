//! QuietCraft: Original automated silence detection, derushing, and audio timeline trimmer
//! for EffectCraft.
//!
//! Provides pure-Rust algorithms for windowed RMS decibel calculation, noise floor thresholding,
//! cadence filtering, attack/release padding, and ripple-cut timeline retiming.

pub mod audio;

use serde::{Deserialize, Serialize};

/// Type of audio timeline segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentKind {
    Speech,
    Silence,
}

/// A contiguous time segment on the audio track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub id: usize,
    pub start: f64,
    pub end: f64,
    pub kind: SegmentKind,
}

impl Segment {
    pub fn new(id: usize, start: f64, end: f64, kind: SegmentKind) -> Self {
        Self {
            id,
            start: start.max(0.0),
            end: end.max(start),
            kind,
        }
    }

    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }
}

/// Parameters controlling silence detection and boundary padding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuietParams {
    /// Noise threshold in decibels full-scale (dBFS).
    /// Values below this are classified as silence (e.g. -35.0 dB).
    pub threshold_db: f64,

    /// Minimum silence duration in seconds.
    /// Silences shorter than this are preserved as natural speech pauses.
    pub min_silence_dur: f64,

    /// Minimum speech duration in seconds.
    /// Audio bursts shorter than this inside silence are suppressed as noise.
    pub min_speech_dur: f64,

    /// Pre-roll (attack margin) in seconds.
    /// Extends speech start earlier so leading consonant sounds are not clipped.
    pub pre_roll_pad: f64,

    /// Post-roll (release margin) in seconds.
    /// Extends speech end later to preserve natural vowel trails and room decay.
    pub post_roll_pad: f64,
}

impl Default for QuietParams {
    fn default() -> Self {
        Self {
            threshold_db: -35.0,
            min_silence_dur: 0.40,
            min_speech_dur: 0.15,
            pre_roll_pad: 0.08,
            post_roll_pad: 0.12,
        }
    }
}

/// Presets tailored for common editing workflows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: String,
    pub params: QuietParams,
}

/// Standard built-in presets.
pub fn built_in_presets() -> Vec<Preset> {
    vec![
        Preset {
            name: "Default Podcast".into(),
            description: "Balanced silence cuts preserving natural conversational flow".into(),
            params: QuietParams {
                threshold_db: -35.0,
                min_silence_dur: 0.40,
                min_speech_dur: 0.15,
                pre_roll_pad: 0.08,
                post_roll_pad: 0.12,
            },
        },
        Preset {
            name: "Aggressive Jump-Cut".into(),
            description: "Tight, fast-paced YouTube/vlog jump cuts with minimal pauses".into(),
            params: QuietParams {
                threshold_db: -30.0,
                min_silence_dur: 0.25,
                min_speech_dur: 0.10,
                pre_roll_pad: 0.04,
                post_roll_pad: 0.06,
            },
        },
        Preset {
            name: "Noisy Environment".into(),
            description: "Higher noise-floor cutoff for rooms with fan/air conditioning hum".into(),
            params: QuietParams {
                threshold_db: -26.0,
                min_silence_dur: 0.45,
                min_speech_dur: 0.18,
                pre_roll_pad: 0.10,
                post_roll_pad: 0.15,
            },
        },
        Preset {
            name: "Gentle Lecture".into(),
            description: "Preserves longer reflective pauses for educational/lecture content".into(),
            params: QuietParams {
                threshold_db: -40.0,
                min_silence_dur: 0.65,
                min_speech_dur: 0.20,
                pre_roll_pad: 0.12,
                post_roll_pad: 0.18,
            },
        },
    ]
}

/// Energy measurement of an audio time slice.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowEnergy {
    pub time: f64,
    pub rms: f64,
    pub db: f64,
}

/// Ripple-cut information for moving a speech segment forward on the timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RippleCut {
    pub index: usize,
    pub source_in: f64,
    pub source_out: f64,
    pub timeline_start: f64,
    pub timeline_end: f64,
    pub duration: f64,
}

/// Comprehensive silence analysis report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilenceReport {
    pub total_duration: f64,
    pub tightened_duration: f64,
    pub silence_removed: f64,
    pub time_saved_pct: f64,
    pub speech_segments: Vec<Segment>,
    pub silence_segments: Vec<Segment>,
    pub ripple_cuts: Vec<RippleCut>,
    pub cut_count: usize,
}

// --- Mathematical Audio Analysis ---

/// Calculate Root Mean Square (RMS) of float audio samples.
pub fn calculate_rms(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut sum_sq = 0.0f64;
    for &s in samples {
        let v = s as f64;
        sum_sq += v * v;
    }
    (sum_sq / (samples.len() as f64)).sqrt()
}

/// Convert normalized RMS amplitude (0.0 .. 1.0) to decibels full-scale (dBFS).
pub fn rms_to_db(rms: f64) -> f64 {
    if rms <= 1e-6 {
        -120.0 // Noise floor baseline
    } else {
        (20.0 * rms.log10()).max(-120.0)
    }
}

/// Slice an audio buffer into short analysis windows and calculate their RMS energy in dBFS.
pub fn analyze_energy_windows(
    samples: &[f32],
    sample_rate: u32,
    window_sec: f64,
    hop_sec: f64,
) -> Vec<WindowEnergy> {
    if samples.is_empty() || sample_rate == 0 {
        return Vec::new();
    }

    let win_size = ((window_sec * sample_rate as f64).round() as usize).max(1);
    let hop_size = ((hop_sec * sample_rate as f64).round() as usize).max(1);

    let mut out = Vec::new();
    let mut pos = 0;

    while pos < samples.len() {
        let end = (pos + win_size).min(samples.len());
        let chunk = &samples[pos..end];
        let rms = calculate_rms(chunk);
        let db = rms_to_db(rms);
        let time = pos as f64 / sample_rate as f64;

        out.push(WindowEnergy { time, rms, db });
        pos += hop_size;
    }

    out
}

/// Detect silence and speech segments from windowed energy data using thresholding,
/// cadence filters, and attack/release padding.
pub fn detect_silence(
    windows: &[WindowEnergy],
    total_duration: f64,
    params: &QuietParams,
) -> SilenceReport {
    if windows.is_empty() || total_duration <= 0.0 {
        return SilenceReport {
            total_duration,
            tightened_duration: total_duration,
            silence_removed: 0.0,
            time_saved_pct: 0.0,
            speech_segments: vec![Segment::new(1, 0.0, total_duration, SegmentKind::Speech)],
            silence_segments: Vec::new(),
            ripple_cuts: vec![RippleCut {
                index: 1,
                source_in: 0.0,
                source_out: total_duration,
                timeline_start: 0.0,
                timeline_end: total_duration,
                duration: total_duration,
            }],
            cut_count: 0,
        };
    }

    let _hop_sec = if windows.len() > 1 {
        windows[1].time - windows[0].time
    } else {
        total_duration
    };

    // Step 1: Raw per-window classification
    let mut raw_speech: Vec<(f64, f64)> = Vec::new(); // (start, end)
    let mut in_speech = false;
    let mut current_start = 0.0;

    for w in windows {
        let is_speech = w.db > params.threshold_db;
        if is_speech {
            if !in_speech {
                in_speech = true;
                current_start = w.time;
            }
        } else if in_speech {
            in_speech = false;
            raw_speech.push((current_start, w.time));
        }
    }
    if in_speech {
        raw_speech.push((current_start, total_duration));
    }

    // If completely silent
    if raw_speech.is_empty() {
        let full_silence = Segment::new(1, 0.0, total_duration, SegmentKind::Silence);
        return SilenceReport {
            total_duration,
            tightened_duration: 0.0,
            silence_removed: total_duration,
            time_saved_pct: 100.0,
            speech_segments: Vec::new(),
            silence_segments: vec![full_silence],
            ripple_cuts: Vec::new(),
            cut_count: 1,
        };
    }

    // Step 2: Suppress brief speech bursts that are shorter than min_speech_dur
    let mut filtered_speech: Vec<(f64, f64)> = Vec::new();
    for (st, en) in raw_speech {
        if (en - st) >= params.min_speech_dur {
            filtered_speech.push((st, en));
        }
    }

    if filtered_speech.is_empty() {
        // All bursts were below min_speech_dur (e.g. isolated mouth clicks in silence)
        let full_silence = Segment::new(1, 0.0, total_duration, SegmentKind::Silence);
        return SilenceReport {
            total_duration,
            tightened_duration: 0.0,
            silence_removed: total_duration,
            time_saved_pct: 100.0,
            speech_segments: Vec::new(),
            silence_segments: vec![full_silence],
            ripple_cuts: Vec::new(),
            cut_count: 1,
        };
    }

    // Step 3: Merge speech across micro-pauses shorter than min_silence_dur
    let mut bridged_speech: Vec<(f64, f64)> = Vec::new();
    for (st, en) in filtered_speech {
        if let Some(last) = bridged_speech.last_mut() {
            let gap = st - last.1;
            if gap < params.min_silence_dur {
                last.1 = en; // Merge across short pause
                continue;
            }
        }
        bridged_speech.push((st, en));
    }

    // Step 4: Apply attack (pre-roll) and release (post-roll) padding
    let mut padded_speech: Vec<(f64, f64)> = Vec::new();
    for (st, en) in bridged_speech {
        let p_start = (st - params.pre_roll_pad).max(0.0);
        let p_end = (en + params.post_roll_pad).min(total_duration);
        padded_speech.push((p_start, p_end));
    }

    // Step 5: Merge overlapping or contiguous speech intervals resulting from padding
    let mut merged_speech: Vec<Segment> = Vec::new();
    for (st, en) in padded_speech {
        if let Some(last) = merged_speech.last_mut() {
            if st <= last.end {
                last.end = last.end.max(en);
                continue;
            }
        }
        merged_speech.push(Segment::new(merged_speech.len() + 1, st, en, SegmentKind::Speech));
    }

    // Re-index speech segments
    for (idx, seg) in merged_speech.iter_mut().enumerate() {
        seg.id = idx + 1;
    }

    // Step 6: Derive silence intervals from gaps
    let mut silence_segments: Vec<Segment> = Vec::new();
    let mut cursor = 0.0;
    let mut s_idx = 1;

    for sp in &merged_speech {
        if sp.start > cursor + 1e-4 {
            silence_segments.push(Segment::new(s_idx, cursor, sp.start, SegmentKind::Silence));
            s_idx += 1;
        }
        cursor = sp.end;
    }
    if cursor + 1e-4 < total_duration {
        silence_segments.push(Segment::new(s_idx, cursor, total_duration, SegmentKind::Silence));
    }

    // Step 7: Calculate ripple retiming
    let ripple_cuts = calculate_ripple_cuts(&merged_speech);
    let tightened_duration = ripple_cuts.last().map_or(0.0, |r| r.timeline_end);
    let silence_removed = (total_duration - tightened_duration).max(0.0);
    let time_saved_pct = if total_duration > 0.0 {
        (silence_removed / total_duration) * 100.0
    } else {
        0.0
    };
    let cut_count = silence_segments.len();

    SilenceReport {
        total_duration,
        tightened_duration,
        silence_removed,
        time_saved_pct,
        speech_segments: merged_speech,
        silence_segments,
        ripple_cuts,
        cut_count,
    }
}

/// Calculate sequential timeline placements for speech segments with silence removed.
pub fn calculate_ripple_cuts(speech_segments: &[Segment]) -> Vec<RippleCut> {
    let mut cuts = Vec::new();
    let mut timeline_cursor = 0.0;

    for (i, seg) in speech_segments.iter().enumerate() {
        let dur = seg.duration();
        let end = timeline_cursor + dur;
        cuts.push(RippleCut {
            index: i + 1,
            source_in: seg.start,
            source_out: seg.end,
            timeline_start: timeline_cursor,
            timeline_end: end,
            duration: dur,
        });
        timeline_cursor = end;
    }

    cuts
}

// --- Exporting Utilities ---

/// Export cut list to open JSON format.
pub fn export_json(report: &SilenceReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
}

/// Export cut list to CSV.
pub fn export_csv(report: &SilenceReport) -> String {
    let mut out = String::from("Index,Type,Source_In,Source_Out,Duration_Seconds\n");
    for s in &report.speech_segments {
        out.push_str(&format!("{},Speech,{:.3},{:.3},{:.3}\n", s.id, s.start, s.end, s.duration()));
    }
    for s in &report.silence_segments {
        out.push_str(&format!("{},Silence,{:.3},{:.3},{:.3}\n", s.id, s.start, s.end, s.duration()));
    }
    out
}

/// Format seconds to standard SMPTE timecode (HH:MM:SS:FF at 30 fps).
fn format_timecode(seconds: f64, fps: f64) -> String {
    let total_frames = (seconds * fps).round() as u64;
    let frames = total_frames % (fps.round() as u64);
    let total_secs = total_frames / (fps.round() as u64);
    let s = total_secs % 60;
    let total_mins = total_secs / 60;
    let m = total_mins % 60;
    let h = total_mins / 60;
    format!("{h:02}:{m:02}:{s:02}:{frames:02}")
}

/// Export CMX 3600 style EDL for NLE interchange.
pub fn export_edl(report: &SilenceReport, title: &str) -> String {
    let fps = 30.0;
    let mut edl = format!("TITLE: {title}\nFCM: NON-DROP FRAME\n\n");

    for cut in &report.ripple_cuts {
        let src_in = format_timecode(cut.source_in, fps);
        let src_out = format_timecode(cut.source_out, fps);
        let dst_in = format_timecode(cut.timeline_start, fps);
        let dst_out = format_timecode(cut.timeline_end, fps);

        edl.push_str(&format!(
            "{:03}  AX       AA/V  C        {} {} {} {}\n* FROM CLIP: Speech Segment #{}\n\n",
            cut.index, src_in, src_out, dst_in, dst_out, cut.index
        ));
    }

    edl
}
