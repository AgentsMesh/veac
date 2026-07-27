use std::path::Path;
use std::process::Command;

use serde_json::Value;

pub(super) fn codec(path: &Path) -> String {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "s:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert_success(&output);
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["streams"][0]["codec_name"]
        .as_str()
        .unwrap()
        .to_owned()
}

pub(super) fn frame_digest(sidecar: Option<&Path>, width: u32, height: u32) -> String {
    let mut command = Command::new("ffmpeg");
    command.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        &format!("color=c=black:s={width}x{height}:r=25:d=1"),
    ]);
    if let Some(path) = sidecar {
        command.args(["-vf", &format!("ass=filename={}", path.display())]);
    }
    let output = command
        .args(["-frames:v", "1", "-f", "framemd5", "-"])
        .output()
        .unwrap();
    assert_success(&output);
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .find(|line| !line.starts_with('#') && !line.trim().is_empty())
        .unwrap()
        .rsplit(',')
        .next()
        .unwrap()
        .trim()
        .to_owned()
}

fn assert_success(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
