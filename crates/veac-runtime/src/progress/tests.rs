use super::*;

#[test]
fn parses_complete_progress_line() {
    let progress = parse_ffmpeg_progress(
        "frame=  42 fps=29.97 q=22.0 time=01:02:03.50 bitrate=1000kbits/s speed=1.25x",
    )
    .expect("progress line");
    assert_eq!(progress.frame, 42);
    assert!((progress.fps - 29.97).abs() < 0.001);
    assert!((progress.time_sec - 3723.5).abs() < 0.001);
    assert!((progress.speed - 1.25).abs() < 0.001);
}

#[test]
fn defaults_malformed_optional_values() {
    let progress = parse_ffmpeg_progress("frame= nope fps=bad time=not-a-time speed=fastx")
        .expect("required keys are present");
    assert_eq!(progress.frame, 0);
    assert_eq!(progress.fps, 0.0);
    assert_eq!(progress.time_sec, 0.0);
    assert_eq!(progress.speed, 0.0);
}

#[test]
fn rejects_non_progress_lines_and_parses_plain_seconds() {
    assert!(parse_ffmpeg_progress("frame=1 fps=30").is_none());
    assert!(parse_ffmpeg_progress("time=00:00:01.0 speed=1x").is_none());
    assert_eq!(parse_time_to_seconds("2.75"), 2.75);
    assert_eq!(parse_time_to_seconds("broken"), 0.0);
    assert_eq!(parse_time_to_seconds("01:bad:03"), 3603.0);
}

#[test]
fn extract_value_handles_missing_keys_and_terminal_values() {
    assert_eq!(extract_value("frame=12", "frame="), Some("12"));
    assert_eq!(extract_value("frame=12 fps=30", "fps="), Some("30"));
    assert_eq!(extract_value("frame=12", "time="), None);
}
