use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn asymmetric_vfr_fixture(directory: &Path) -> PathBuf {
    fixture(
        directory.join("asymmetric-vfr.mp4"),
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc2=s=96x54:r=10:d=1.1",
            "-vf",
            "select='eq(n,0)+eq(n,1)+eq(n,4)+eq(n,9)',setpts=N*N/10/TB",
            "-fps_mode",
            "vfr",
            "-c:v",
            "libx264",
            "-bf",
            "0",
            "-pix_fmt",
            "yuv420p",
        ],
    )
}

pub(super) fn equal_rate_vfr_fixture(directory: &Path) -> PathBuf {
    fixture(
        directory.join("equal-summary-rate-vfr.mkv"),
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc2=s=96x54:r=20:d=0.4",
            "-vf",
            r"settb=1/1000,setpts=floor(N/2)*200+mod(N\,2)*50",
            "-fps_mode",
            "passthrough",
            "-c:v",
            "ffv1",
        ],
    )
}

pub(super) fn fractional_cfr_fixture(directory: &Path) -> PathBuf {
    fixture(
        directory.join("fractional-cfr.mkv"),
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc2=s=96x54:r=30000/1001:d=0.5",
            "-c:v",
            "ffv1",
        ],
    )
}

fn fixture(output: PathBuf, arguments: &[&str]) -> PathBuf {
    let result = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(arguments)
        .arg(&output)
        .output()
        .expect("start cadence fixture render");
    assert!(
        result.status.success(),
        "cadence fixture failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}

pub(super) fn stream_rates(path: &Path) -> String {
    text_probe(path, "stream=avg_frame_rate,r_frame_rate")
        .trim()
        .to_owned()
}

pub(super) fn frame_pts(path: &Path) -> Vec<f64> {
    text_probe(path, "frame=pts_time")
        .lines()
        .map(|line| line.trim_end_matches(',').parse().unwrap())
        .collect()
}

pub(super) fn packet_ticks(path: &Path) -> Vec<i64> {
    text_probe(path, "packet=pts")
        .lines()
        .map(|line| line.trim_end_matches(',').parse().unwrap())
        .collect()
}

fn text_probe(path: &Path, entries: &str) -> String {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            entries,
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .expect("start cadence timestamp probe");
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}
