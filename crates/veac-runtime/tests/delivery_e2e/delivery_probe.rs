use std::process::Command;

use serde_json::Value;

use super::support::*;

pub(super) fn stream(path: &Path, selector: &str, fields: &str) -> Value {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            selector,
            "-show_entries",
            fields,
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .expect("run ffprobe");
    assert!(
        output.status.success(),
        "ffprobe failed for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).expect("ffprobe JSON")["streams"][0].clone()
}

pub(super) fn rgb(path: &Path) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args(["-frames:v", "1", "-pix_fmt", "rgb24", "-f", "rawvideo", "-"])
        .output()
        .expect("decode RGB frame");
    assert!(
        output.status.success(),
        "decode failed for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

pub(super) fn gif_loop_count(path: &Path) -> u16 {
    let bytes = std::fs::read(path).expect("read GIF");
    let marker = b"NETSCAPE2.0";
    let index = bytes
        .windows(marker.len())
        .position(|window| window == marker)
        .expect("GIF loop extension");
    let loop_block = bytes
        .get(index + marker.len()..index + marker.len() + 5)
        .expect("complete GIF loop extension");
    assert_eq!(loop_block[0], 3);
    assert_eq!(loop_block[1], 1);
    assert_eq!(loop_block[4], 0);
    u16::from_le_bytes([loop_block[2], loop_block[3]])
}
