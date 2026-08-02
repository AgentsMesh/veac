use std::{path::Path, process::Command};

use super::*;

pub(crate) fn layout_video_fixture(dir: &Path) -> PathBuf {
    let output = dir.join("layout-source.mp4");
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=s=192x54:r=10:d=2",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&output)
        .status()
        .expect("start FFmpeg layout fixture");
    assert!(status.success(), "FFmpeg layout fixture failed");
    output
}
