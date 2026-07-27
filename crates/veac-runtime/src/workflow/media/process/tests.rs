#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use super::*;

#[test]
fn timeout_kills_ffmpeg_descendants_before_they_can_escape() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("descendant-ran");
    let script = temp.path().join("ffmpeg-tree.sh");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\n(sleep 1; touch '{}') &\nsleep 5\n",
            marker.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();

    let error = run(
        &script,
        &[],
        &temp.path().join("output"),
        MediaArtifactLimits::default(),
        Instant::now() + Duration::from_millis(100),
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    std::thread::sleep(Duration::from_millis(1_200));
    assert!(!marker.exists());
}

#[test]
fn output_monitor_error_also_kills_ffmpeg_descendants() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("descendant-ran");
    let output = temp.path().join("output");
    let script = temp.path().join("ffmpeg-monitor.sh");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\nmkdir '{}'\n(sleep 2; touch '{}') &\nsleep 5\n",
            output.display(),
            marker.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();

    assert!(run(
        &script,
        &[],
        &output,
        MediaArtifactLimits::default(),
        Instant::now() + Duration::from_secs(5),
    )
    .is_err());
    std::thread::sleep(Duration::from_millis(2_200));
    assert!(!marker.exists());
}
