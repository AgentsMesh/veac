#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use super::*;

fn run(
    executable: &std::path::Path,
    arguments: &[std::ffi::OsString],
    output: &std::path::Path,
    limits: MediaArtifactLimits,
    deadline: Instant,
) -> WorkflowResult<()> {
    run_while(executable, arguments, output, limits, deadline, || true)
}

#[test]
fn missing_executable_is_a_stable_tool_start_failure() {
    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("missing-ffmpeg");
    let error = run(
        &executable,
        &[],
        &temp.path().join("output"),
        MediaArtifactLimits::default(),
        Instant::now() + Duration::from_secs(1),
    )
    .unwrap_err();

    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
    assert_eq!(
        error.to_string(),
        "failed to start FFmpeg artifact derivation"
    );
    let source = std::error::Error::source(&error).unwrap();
    assert_eq!(
        source.downcast_ref::<std::io::Error>().unwrap().kind(),
        std::io::ErrorKind::NotFound
    );
}

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

#[test]
fn cancelled_guard_kills_ffmpeg_descendants() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("descendant-ran");
    let script = temp.path().join("ffmpeg-cancel.sh");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\n(sleep 1; touch '{}') &\nsleep 5\n",
            marker.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    let mut polls = 0;
    let error = run_while(
        &script,
        &[],
        &temp.path().join("output"),
        MediaArtifactLimits::default(),
        Instant::now() + Duration::from_secs(5),
        || {
            polls += 1;
            polls < 3
        },
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(error.to_string().contains("cancelled"));
    std::thread::sleep(Duration::from_millis(1_200));
    assert!(!marker.exists());
}
