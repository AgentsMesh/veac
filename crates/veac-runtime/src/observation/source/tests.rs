use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use tempfile::{tempdir, TempDir};

use super::*;
use crate::asset::sha256_identity;

#[test]
fn verify_accepts_the_binding_then_rejects_changed_content() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("source.bin");
    fs::write(&path, b"before").unwrap();
    let source = bound(&path, None);
    verify(&source).unwrap();

    fs::write(&path, b"after!").unwrap();
    let error = verify(&source).unwrap_err();
    assert!(error.message.contains("media observation source changed"));
    assert!(error.message.contains("pinned SHA-256 identity"));
}

#[test]
fn probe_honors_explicit_video_stream_and_maps_selection_errors() {
    let (_temp, path) = multi_video_fixture();
    let explicit = bound(&path, Some(1));
    let ffprobe = crate::asset::SystemFfprobe::default();
    let snapshot = probe(&ffprobe, &explicit, deadline()).unwrap();
    let selection = snapshot.selected_video_stream.unwrap();
    assert_eq!((selection.global_index, selection.type_index), (1, 1));
    assert!(snapshot.selected_audio_stream.is_none());

    let mut mismatched = snapshot.clone();
    mismatched.observed_identity.digest = "0".repeat(64);
    assert_eq!(
        verify_probe_identity(&mismatched, &explicit)
            .unwrap_err()
            .message,
        "media observation source identity does not match its binding"
    );

    let error = probe(&ffprobe, &bound(&path, Some(99)), deadline()).unwrap_err();
    assert!(error.message.contains("media observation probe failed"));
    assert!(
        error
            .message
            .contains("stream 99 cannot be selected as Video"),
        "{}",
        error.message
    );
}

fn bound(path: &Path, video_stream: Option<u32>) -> ObservationSource {
    ObservationSource {
        identity: sha256_identity(path).unwrap(),
        path: path.to_path_buf(),
        video_stream,
    }
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

fn multi_video_fixture() -> (TempDir, PathBuf) {
    let temp = tempdir().unwrap();
    let path = temp.path().join("streams.mkv");
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=4x2:r=1:d=1",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=4x2:r=1:d=1",
            "-map",
            "0:v:0",
            "-map",
            "1:v:0",
            "-c:v",
            "ffv1",
            "-threads",
            "1",
            "-y",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    (temp, path)
}
