use veac_runtime::progress::parse_ffmpeg_progress;

#[test]
fn test_parse_progress_line() {
    let line = "frame=  100 fps=30.0 q=28.0 size=    256kB time=00:00:03.33 bitrate= 629.4kbits/s speed=1.00x";
    let progress = parse_ffmpeg_progress(line).unwrap();
    assert_eq!(progress.frame, 100);
    assert!((progress.fps - 30.0).abs() < 0.1);
    assert!((progress.time_sec - 3.33).abs() < 0.01);
    assert!((progress.speed - 1.0).abs() < 0.01);
}

#[test]
fn test_parse_non_progress_line() {
    let line = "Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'input.mp4':";
    assert!(parse_ffmpeg_progress(line).is_none());
}

#[test]
fn test_check_ffmpeg_installed() {
    let version = veac_runtime::executor::check_ffmpeg()
        .expect("FFmpeg is required by the integration test environment");
    assert!(version.contains("ffmpeg"));
}
