#![cfg(unix)]

use std::ffi::OsString;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::*;

#[test]
fn preflight_deadline_and_cancellation_stop_before_process_launch() {
    let missing = Path::new("/veac/missing/ffmpeg");
    let output = Path::new("/veac/missing/output");
    let expired = run_while(
        missing,
        &[],
        output,
        MediaArtifactLimits::default(),
        Instant::now(),
        || true,
    )
    .unwrap_err();
    assert_eq!(expired.kind, WorkflowErrorKind::ResourceLimit);
    let cancelled = run_while(
        missing,
        &[],
        output,
        MediaArtifactLimits::default(),
        deadline(),
        || false,
    )
    .unwrap_err();
    assert!(cancelled.to_string().contains("cancelled"));
}

#[test]
fn helper_paths_enforce_io_size_and_diagnostic_limits() {
    let temp = tempfile::tempdir().unwrap();
    let parent = temp.path().join("parent");
    std::fs::write(&parent, b"file").unwrap();
    assert_eq!(
        output_size(&parent.join("child")).unwrap_err().kind,
        WorkflowErrorKind::Io
    );

    let output = temp.path().join("output");
    std::fs::write(&output, b"x").unwrap();
    assert_eq!(
        verify_output(&output, 0).unwrap_err().kind,
        WorkflowErrorKind::ResourceLimit
    );

    let mut stderr = tempfile::tempfile().unwrap();
    stderr
        .write_all(&vec![b'x'; (MAX_STDERR_BYTES + 1) as usize])
        .unwrap();
    assert_eq!(
        detail(&mut stderr).unwrap(),
        "stderr exceeded the execution budget"
    );
}

#[test]
fn active_process_enforces_stderr_output_and_output_path_limits() {
    let temp = tempfile::tempdir().unwrap();
    let stderr = script(temp.path(), "head -c 1048577 /dev/zero >&2; sleep 5");
    assert_limit(
        &stderr,
        &[],
        &temp.path().join("stderr-output"),
        default_limits(),
    );

    let output = temp.path().join("payload");
    let payload = script(temp.path(), "printf xx > \"$1\"; sleep 5");
    assert_limit(
        &payload,
        &[output.as_os_str().to_owned()],
        &output,
        payload_limits(),
    );

    let parent = temp.path().join("not-a-directory");
    std::fs::write(&parent, b"file").unwrap();
    let waiting = script(temp.path(), "sleep 5");
    let error = run_while(
        &waiting,
        &[],
        &parent.join("output"),
        default_limits(),
        deadline(),
        || true,
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::Io);
}

fn assert_limit(
    executable: &Path,
    arguments: &[OsString],
    output: &Path,
    limits: MediaArtifactLimits,
) {
    let error = run_while(executable, arguments, output, limits, deadline(), || true).unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
}

fn script(root: &Path, body: &str) -> PathBuf {
    let path = root.join(format!("process-{}.sh", body.len()));
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}

fn default_limits() -> MediaArtifactLimits {
    MediaArtifactLimits::default()
}

fn payload_limits() -> MediaArtifactLimits {
    MediaArtifactLimits {
        max_payload_bytes: 1,
        ..MediaArtifactLimits::default()
    }
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(3)
}
