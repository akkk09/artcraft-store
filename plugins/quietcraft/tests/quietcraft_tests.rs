//! Comprehensive unit tests for QuietCraft silence detection, padding, retiming, and audio parsing.

use quietcraft::audio::{create_test_wav, generate_synthetic_audio, parse_wav};
use quietcraft::{
    analyze_energy_windows, calculate_ripple_cuts, calculate_rms, detect_silence, export_edl,
    export_json, rms_to_db, QuietParams, SegmentKind,
};

#[test]
fn test_rms_and_db_calculations() {
    let zeros = vec![0.0f32; 1000];
    let rms_zero = calculate_rms(&zeros);
    assert_eq!(rms_zero, 0.0);
    assert_eq!(rms_to_db(rms_zero), -120.0);

    // Full scale DC (1.0)
    let full = vec![1.0f32; 1000];
    let rms_full = calculate_rms(&full);
    assert!((rms_full - 1.0).abs() < 1e-5);
    assert!((rms_to_db(rms_full) - 0.0).abs() < 1e-4);

    // Half scale DC (0.5) => ~ -6.02 dB
    let half = vec![0.5f32; 1000];
    let rms_half = calculate_rms(&half);
    assert!((rms_half - 0.5).abs() < 1e-5);
    assert!((rms_to_db(rms_half) - (-6.0205)).abs() < 0.01);
}

#[test]
fn test_synthetic_audio_and_wav_roundtrip() {
    let rate = 44100;
    // 0.5s tone (amp 0.8), 0.5s silence, 0.5s tone (amp 0.8)
    let samples = generate_synthetic_audio(rate, &[(0.5, 0.8), (0.5, 0.0), (0.5, 0.8)]);
    assert_eq!(samples.len(), (1.5 * rate as f64).round() as usize);

    let wav_bytes = create_test_wav(rate, &samples);
    assert!(wav_bytes.len() > 44);

    let parsed = parse_wav(&wav_bytes).expect("Failed to parse generated WAV");
    assert_eq!(parsed.sample_rate, rate);
    assert_eq!(parsed.channels, 1);
    assert!((parsed.duration - 1.5).abs() < 0.01);
    assert_eq!(parsed.samples.len(), samples.len());

    // Check first sample of speech is loud, middle is silence
    let speech_rms = calculate_rms(&parsed.samples[0..1000]);
    let silence_rms = calculate_rms(&parsed.samples[(0.6 * rate as f64) as usize..(0.9 * rate as f64) as usize]);
    assert!(speech_rms > 0.4);
    assert!(silence_rms < 0.001);
}

#[test]
fn test_silence_detection_basic() {
    let rate = 44100;
    // 1.0s speech, 1.0s silence, 1.0s speech
    let samples = generate_synthetic_audio(rate, &[(1.0, 0.6), (1.0, 0.0), (1.0, 0.6)]);
    let total_dur = 3.0;

    let windows = analyze_energy_windows(&samples, rate, 0.025, 0.010);
    assert!(!windows.is_empty());

    let params = QuietParams {
        threshold_db: -35.0,
        min_silence_dur: 0.30,
        min_speech_dur: 0.15,
        pre_roll_pad: 0.0,
        post_roll_pad: 0.0,
    };

    let report = detect_silence(&windows, total_dur, &params);
    assert_eq!(report.speech_segments.len(), 2, "Expected 2 speech segments");
    assert_eq!(report.silence_segments.len(), 1, "Expected 1 silence segment");

    let speech1 = &report.speech_segments[0];
    let silence = &report.silence_segments[0];
    let speech2 = &report.speech_segments[1];

    assert!((speech1.start - 0.0).abs() < 0.05);
    assert!((speech1.end - 1.0).abs() < 0.05);
    assert!((silence.start - 1.0).abs() < 0.05);
    assert!((silence.end - 2.0).abs() < 0.05);
    assert!((speech2.start - 2.0).abs() < 0.05);
    assert!((speech2.end - 3.0).abs() < 0.05);

    assert!((report.silence_removed - 1.0).abs() < 0.1);
    assert!((report.tightened_duration - 2.0).abs() < 0.1);
    assert!((report.time_saved_pct - 33.3).abs() < 3.0);
}

#[test]
fn test_min_silence_duration_bridges_micropauses() {
    let rate = 44100;
    // 1.0s speech, 0.2s quick breath (silence), 1.0s speech
    let samples = generate_synthetic_audio(rate, &[(1.0, 0.6), (0.2, 0.0), (1.0, 0.6)]);
    let windows = analyze_energy_windows(&samples, rate, 0.025, 0.010);

    // Min silence is 0.40s, so the 0.20s pause should be bridged and kept!
    let params = QuietParams {
        threshold_db: -35.0,
        min_silence_dur: 0.40,
        min_speech_dur: 0.10,
        pre_roll_pad: 0.0,
        post_roll_pad: 0.0,
    };

    let report = detect_silence(&windows, 2.2, &params);
    assert_eq!(report.speech_segments.len(), 1, "Short pause should be bridged into 1 continuous speech segment");
    assert_eq!(report.silence_segments.len(), 0);
    assert!((report.tightened_duration - 2.2).abs() < 0.05);
}

#[test]
fn test_min_speech_duration_suppresses_mouth_clicks() {
    let rate = 44100;
    // 1.0s silence, 0.05s tiny click (amp 0.6), 1.0s silence
    let samples = generate_synthetic_audio(rate, &[(1.0, 0.0), (0.05, 0.6), (1.0, 0.0)]);
    let windows = analyze_energy_windows(&samples, rate, 0.025, 0.010);

    let params = QuietParams {
        threshold_db: -35.0,
        min_silence_dur: 0.30,
        min_speech_dur: 0.15, // Clicks < 150ms ignored
        pre_roll_pad: 0.0,
        post_roll_pad: 0.0,
    };

    let report = detect_silence(&windows, 2.05, &params);
    // Click should be suppressed; track treated as fully silent
    assert_eq!(report.silence_segments.len(), 1);
    assert_eq!(report.speech_segments.len(), 0);
}

#[test]
fn test_pre_and_post_roll_padding() {
    let rate = 44100;
    // 0.5s silence, 1.0s speech (at 0.5s to 1.5s), 0.5s silence (total 2.0s)
    let samples = generate_synthetic_audio(rate, &[(0.5, 0.0), (1.0, 0.7), (0.5, 0.0)]);
    let windows = analyze_energy_windows(&samples, rate, 0.025, 0.010);

    let params = QuietParams {
        threshold_db: -35.0,
        min_silence_dur: 0.20,
        min_speech_dur: 0.10,
        pre_roll_pad: 0.08,  // 80ms attack
        post_roll_pad: 0.12, // 120ms release
    };

    let report = detect_silence(&windows, 2.0, &params);
    assert_eq!(report.speech_segments.len(), 1);
    let sp = &report.speech_segments[0];

    // Speech was 0.5s to 1.5s. With padding:
    // start should be ~ 0.5 - 0.08 = 0.42s
    // end should be ~ 1.5 + 0.12 = 1.62s
    assert!((sp.start - 0.42).abs() < 0.05, "Padded start was {}", sp.start);
    assert!((sp.end - 1.62).abs() < 0.05, "Padded end was {}", sp.end);
}

#[test]
fn test_ripple_cut_timeline_retiming() {
    let segments = vec![
        quietcraft::Segment::new(1, 0.5, 1.5, SegmentKind::Speech), // 1.0s dur
        quietcraft::Segment::new(2, 3.0, 4.2, SegmentKind::Speech), // 1.2s dur
        quietcraft::Segment::new(3, 7.0, 8.0, SegmentKind::Speech), // 1.0s dur
    ];

    let cuts = calculate_ripple_cuts(&segments);
    assert_eq!(cuts.len(), 3);

    // Cut 1: timeline 0.0 -> 1.0
    assert_eq!(cuts[0].source_in, 0.5);
    assert_eq!(cuts[0].source_out, 1.5);
    assert_eq!(cuts[0].timeline_start, 0.0);
    assert_eq!(cuts[0].timeline_end, 1.0);

    // Cut 2: timeline 1.0 -> 2.2
    assert_eq!(cuts[1].source_in, 3.0);
    assert_eq!(cuts[1].source_out, 4.2);
    assert_eq!(cuts[1].timeline_start, 1.0);
    assert_eq!(cuts[1].timeline_end, 2.2);

    // Cut 3: timeline 2.2 -> 3.2
    assert_eq!(cuts[2].source_in, 7.0);
    assert_eq!(cuts[2].source_out, 8.0);
    assert_eq!(cuts[2].timeline_start, 2.2);
    assert_eq!(cuts[2].timeline_end, 3.2);
}

#[test]
fn test_edl_and_json_export() {
    let rate = 44100;
    let samples = generate_synthetic_audio(rate, &[(1.0, 0.8), (1.0, 0.0), (1.0, 0.8)]);
    let windows = analyze_energy_windows(&samples, rate, 0.025, 0.010);
    let params = QuietParams::default();

    let report = detect_silence(&windows, 3.0, &params);
    let json_str = export_json(&report);
    assert!(json_str.contains("speech_segments"));
    assert!(json_str.contains("ripple_cuts"));

    let edl = export_edl(&report, "Interview-Raw");
    assert!(edl.contains("TITLE: Interview-Raw"));
    assert!(edl.contains("FCM: NON-DROP FRAME"));
    assert!(edl.contains("AA/V"));
}
