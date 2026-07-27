use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn attached_picture_fixture(dir: &Path) -> PathBuf {
    let cover = dir.join("cover.png");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=24x24",
            "-frames:v",
            "1",
            "-update",
            "1",
        ])
        .arg(&cover));
    let output = dir.join("attached.mp4");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=64x36:r=10:d=1",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
            "-i",
        ])
        .arg(&cover)
        .args([
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-map",
            "2:v:0",
            "-c:v:0",
            "libx264",
            "-pix_fmt:v:0",
            "yuv420p",
            "-c:a:0",
            "aac",
            "-c:v:1",
            "mjpeg",
            "-disposition:v:0",
            "default",
            "-disposition:v:1",
            "attached_pic",
            "-t",
            "1",
        ])
        .arg(&output));
    output
}

pub(super) fn multiple_video_fixture(dir: &Path) -> PathBuf {
    let output = dir.join("multiple.mp4");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=64x36:r=5:d=1",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=96x54:r=5:d=1",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=880:sample_rate=44100:duration=1",
            "-map",
            "0:v:0",
            "-map",
            "1:v:0",
            "-map",
            "2:a:0",
            "-c:v",
            "libx264",
            "-pix_fmt:v",
            "yuv420p",
            "-c:a",
            "aac",
            "-disposition:v:0",
            "0",
            "-disposition:v:1",
            "default",
            "-t",
            "1",
        ])
        .arg(&output));
    output
}

fn run(command: &mut Command) {
    let output = command.output().expect("start FFmpeg fixture command");
    assert!(
        output.status.success(),
        "FFmpeg fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
