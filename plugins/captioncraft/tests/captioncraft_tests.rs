use captioncraft::{
    parse_srt, parse_vtt, to_srt, to_vtt, parse_timecode,
    check_overlaps, auto_fix_overlaps, shift_timing, split_segment, merge_segments,
    CaptionSegment,
};

#[test]
fn test_parse_and_export_srt() {
    let raw_srt = r#"1
00:00:01,000 --> 00:00:04,500
Welcome to EffectCraft!
This is a captioning test.

2
00:00:05,200 --> 00:00:08,800
Create stunning visual effects
and motion graphics easily.
"#;

    let segs = parse_srt(raw_srt).expect("parse srt");
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].id, 1);
    assert!((segs[0].start_time - 1.0).abs() < 1e-4);
    assert!((segs[0].end_time - 4.5).abs() < 1e-4);
    assert_eq!(segs[0].duration(), 3.5);
    assert_eq!(segs[0].text, "Welcome to EffectCraft!\nThis is a captioning test.");

    assert!((segs[1].start_time - 5.2).abs() < 1e-4);
    assert!((segs[1].end_time - 8.8).abs() < 1e-4);

    let exported = to_srt(&segs);
    assert!(exported.contains("00:00:01,000 --> 00:00:04,500"));
    assert!(exported.contains("Welcome to EffectCraft!"));
}

#[test]
fn test_parse_and_export_vtt() {
    let raw_vtt = r#"WEBVTT - Demo Subtitle Track

1
00:00:02.500 --> 00:00:06.000 line:90% align:center
The quick brown fox jumps

2
00:00:07.100 --> 00:00:10.000
over the lazy dog.
"#;

    let segs = parse_vtt(raw_vtt).expect("parse vtt");
    assert_eq!(segs.len(), 2);
    assert!((segs[0].start_time - 2.5).abs() < 1e-4);
    assert!((segs[0].end_time - 6.0).abs() < 1e-4);
    assert_eq!(segs[0].text, "The quick brown fox jumps");

    let exported = to_vtt(&segs);
    assert!(exported.starts_with("WEBVTT\n\n"));
    assert!(exported.contains("00:00:02.500 --> 00:00:06.000"));
}

#[test]
fn test_unicode_and_rtl_preservation() {
    let raw_srt = "1\n00:00:01,000 --> 00:00:03,000\nمرحبا بك في EffectCraft! 🎬\n\n2\n00:00:03,500 --> 00:00:05,000\nשלום עולם! 日本語と中文\n";
    let segs = parse_srt(raw_srt).expect("parse unicode srt");
    assert_eq!(segs.len(), 2);
    assert!(segs[0].text.contains("مرحبا بك"));
    assert!(segs[0].text.contains("🎬"));
    assert!(segs[1].text.contains("שלום"));
    assert!(segs[1].text.contains("日本語"));

    let exported = to_srt(&segs);
    assert!(exported.contains("مرحبا بك"));
    assert!(exported.contains("🎬"));
}

#[test]
fn test_timecode_parsing_robustness() {
    assert_eq!(parse_timecode("01:02:03.456").unwrap(), 3723.456);
    assert_eq!(parse_timecode("01:02:03,456").unwrap(), 3723.456);
    assert_eq!(parse_timecode("05:20.500").unwrap(), 320.5);
    assert_eq!(parse_timecode("42.5").unwrap(), 42.5);
    assert!(parse_timecode("invalid:time:code:extra").is_err());
}

#[test]
fn test_overlap_detection_and_autofix() {
    let mut segs = vec![
        CaptionSegment::new(1, 1.0, 5.0, "First cue").unwrap(),
        CaptionSegment::new(2, 4.2, 8.0, "Overlapping second cue").unwrap(),
        CaptionSegment::new(3, 9.0, 12.0, "Third non-overlapping cue").unwrap(),
    ];

    let overlaps = check_overlaps(&segs);
    assert_eq!(overlaps.len(), 1);
    assert_eq!(overlaps[0], (0, 1));

    // Fix overlaps with 0.05s gap
    auto_fix_overlaps(&mut segs, 0.05);

    assert!((segs[0].end_time - 4.15).abs() < 1e-4, "End of cue 1 must be trimmed to 4.15s: {}", segs[0].end_time);
    assert_eq!(segs[1].start_time, 4.2);

    let remaining_overlaps = check_overlaps(&segs);
    assert!(remaining_overlaps.is_empty(), "All overlaps must be resolved");
}

#[test]
fn test_shift_timing() {
    let mut segs = vec![
        CaptionSegment::new(1, 2.0, 5.0, "A").unwrap(),
        CaptionSegment::new(2, 6.0, 9.0, "B").unwrap(),
    ];

    // Shift forward by 1.5s
    shift_timing(&mut segs, 1.5);
    assert!((segs[0].start_time - 3.5).abs() < 1e-4);
    assert!((segs[0].end_time - 6.5).abs() < 1e-4);
    assert_eq!(segs[0].duration(), 3.0);

    // Shift backward by 5.0s (clamps at 0.0)
    shift_timing(&mut segs, -5.0);
    assert_eq!(segs[0].start_time, 0.0);
    assert_eq!(segs[0].duration(), 3.0);
}

#[test]
fn test_split_and_merge() {
    let mut segs = vec![
        CaptionSegment::new(1, 0.0, 6.0, "Hello world, this is a test").unwrap(),
        CaptionSegment::new(2, 7.0, 10.0, "Next part").unwrap(),
    ];

    // Split cue 1 at 3.0s
    split_segment(&mut segs, 0, 3.0).expect("split");
    assert_eq!(segs.len(), 3);
    assert_eq!(segs[0].start_time, 0.0);
    assert_eq!(segs[0].end_time, 3.0);
    assert_eq!(segs[1].start_time, 3.0);
    assert_eq!(segs[1].end_time, 6.0);

    // Merge cues 0 and 1 back together
    merge_segments(&mut segs, 0).expect("merge");
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].start_time, 0.0);
    assert_eq!(segs[0].end_time, 6.0);
    assert!(segs[0].text.contains("Hello world"));
}
