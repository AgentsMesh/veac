use std::process::Command;

use super::*;

pub(crate) fn color_tone_fixture(
    directory: &Path,
    name: &str,
    color: &str,
    frequency: u32,
) -> PathBuf {
    let output = directory.join(format!("{name}.mp4"));
    let video = format!("color=c={color}:s={WIDTH}x{HEIGHT}:r={FPS}:d=2");
    let audio = format!("sine=frequency={frequency}:sample_rate=48000:duration=2");
    let result = Command::new("ffmpeg")
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", &video])
        .args(["-f", "lavfi", "-i", &audio])
        .args(["-map", "0:v:0", "-map", "1:a:0", "-t", "2"])
        .args([
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-pix_fmt",
            "yuv420p",
        ])
        .args(["-c:a", "aac", "-b:a", "96k", "-ac", "1"])
        .arg(&output)
        .output()
        .expect("start FFmpeg AV fixture command");
    assert!(
        result.status.success(),
        "FFmpeg AV fixture failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}
