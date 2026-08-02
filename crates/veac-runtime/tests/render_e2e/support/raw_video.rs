use std::process::Command;

use super::*;

pub(crate) fn raw_rgba_frame(path: &Path) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(path)
        .args(["-frames:v", "1", "-pix_fmt", "rgba", "-f", "rawvideo", "-"])
        .output()
        .expect("decode RGBA frame");
    assert!(
        output.status.success(),
        "RGBA decode failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), (WIDTH * HEIGHT * 4) as usize);
    output.stdout
}

pub(crate) fn raw_alpha16_frame(path: &Path) -> Vec<u16> {
    let output = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(path)
        .args([
            "-vf",
            "alphaextract",
            "-frames:v",
            "1",
            "-pix_fmt",
            "gray16le",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()
        .expect("decode 16-bit alpha frame");
    assert!(
        output.status.success(),
        "alpha decode failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
        .stdout
        .chunks_exact(2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .collect()
}
