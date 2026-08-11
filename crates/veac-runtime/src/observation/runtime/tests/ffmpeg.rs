use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::{tempdir, TempDir};

use super::*;
use crate::asset::sha256_identity;

#[test]
fn frame_reports_process_failure_and_an_invalid_raster_size() {
    let (temp, source) = fixture();
    let request = FrameRequest {
        source,
        time: RationalTime::zero(1).unwrap(),
        pixel_format: FramePixelFormat::Rgb8,
    };

    let failure = observer(tool(
        temp.path(),
        "failure",
        "#!/bin/sh\nprintf 'synthetic failure' >&2\nexit 3\n",
    ))
    .frame(&request)
    .unwrap_err();
    assert!(failure.message.contains("synthetic failure"));

    let invalid = observer(tool(
        temp.path(),
        "invalid-raster",
        "#!/bin/sh\nprintf x\nprintf '[Parsed_showinfo_0] n: 0 pts: 0 pts_time:0\\n' >&2\n",
    ))
    .frame(&request)
    .unwrap_err();
    assert_eq!(
        invalid.message,
        "frame observation returned an unexpected raster size"
    );
}

fn observer(path: PathBuf) -> MediaObserver {
    MediaObserver::new(SystemFfmpeg::new(path), ObservationLimits::default()).unwrap()
}

fn fixture() -> (TempDir, ObservationSource) {
    let temp = tempdir().unwrap();
    let path = temp.path().join("fixture.mkv");
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=4x2:r=1:d=1",
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
    let source = ObservationSource {
        identity: sha256_identity(&path).unwrap(),
        path,
        video_stream: None,
    };
    (temp, source)
}

fn tool(root: &Path, name: &str, body: &str) -> PathBuf {
    let path = root.join(name);
    fs::write(&path, body).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&path, permissions).unwrap();
    path
}
