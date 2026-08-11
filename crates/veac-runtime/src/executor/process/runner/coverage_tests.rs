#![cfg(unix)]

use std::io::Write;
use std::process::Command;
use std::time::{Duration, Instant};

use super::*;
use crate::RuntimeErrorKind;

#[test]
fn validate_rejects_every_invalid_resource_policy_field() {
    let root = tempfile::tempdir().unwrap();
    let mut limits = valid_limits(Some(root.path()));
    validate(&limits).unwrap();
    for mutation in [
        |value: &mut ProcessLimits<'_>| value.max_stdout_bytes = 0,
        |value: &mut ProcessLimits<'_>| value.max_stdout_bytes = MAX_CAPTURE_BYTES + 1,
        |value: &mut ProcessLimits<'_>| value.max_stderr_bytes = 0,
        |value: &mut ProcessLimits<'_>| value.max_stderr_bytes = MAX_STDERR_BYTES + 1,
        |value: &mut ProcessLimits<'_>| value.max_output_bytes = 0,
        |value: &mut ProcessLimits<'_>| value.max_output_bytes = MAX_OUTPUT_BYTES + 1,
    ] {
        limits = valid_limits(Some(root.path()));
        mutation(&mut limits);
        assert_eq!(
            validate(&limits).unwrap_err().kind,
            RuntimeErrorKind::General
        );
    }
    limits = valid_limits(Some(root.path()));
    limits.deadline = Instant::now() + MAX_WALL_TIME + Duration::from_secs(1);
    assert_eq!(
        validate(&limits).unwrap_err().kind,
        RuntimeErrorKind::General
    );
}

#[test]
fn tempfile_helpers_recheck_length_before_returning_bytes() {
    let mut exact = tempfile::tempfile().unwrap();
    exact.write_all(b"1234").unwrap();
    assert_eq!(length(&exact).unwrap(), 4);
    assert_eq!(read(&mut exact, 4).unwrap(), b"1234");

    let mut oversized = tempfile::tempfile().unwrap();
    oversized.write_all(b"12345").unwrap();
    assert_eq!(
        read(&mut oversized, 4).unwrap_err().kind,
        RuntimeErrorKind::ResourceLimit
    );
    assert_eq!(
        io_error(std::io::Error::other("io")).kind,
        RuntimeErrorKind::General
    );
}

#[test]
fn resource_violation_checks_streams_and_output_tree_independently() {
    let root = tempfile::tempdir().unwrap();
    let mut stdout = tempfile::tempfile().unwrap();
    let mut stderr = tempfile::tempfile().unwrap();
    let mut limits = valid_limits(Some(root.path()));
    assert_eq!(resource_violation(&stdout, &stderr, &limits).unwrap(), None);

    stdout.write_all(b"12345").unwrap();
    limits.max_stdout_bytes = 4;
    assert_eq!(
        resource_violation(&stdout, &stderr, &limits).unwrap(),
        Some("FFmpeg stdout exceeded its byte limit")
    );
    limits = valid_limits(Some(root.path()));
    stderr.write_all(b"12345").unwrap();
    limits.max_stderr_bytes = 4;
    assert_eq!(
        resource_violation(&tempfile::tempfile().unwrap(), &stderr, &limits).unwrap(),
        Some("FFmpeg stderr exceeded its byte limit")
    );
    limits = valid_limits(Some(root.path()));
    limits.max_output_bytes = 4;
    std::fs::write(root.path().join("output"), b"12345").unwrap();
    assert_eq!(
        resource_violation(
            &tempfile::tempfile().unwrap(),
            &tempfile::tempfile().unwrap(),
            &limits
        )
        .unwrap(),
        Some("FFmpeg output exceeded its byte limit")
    );
}

#[test]
fn stop_helpers_terminate_a_real_process_and_return_original_kind() {
    let mut child = Command::new("/bin/sh")
        .args(["-c", "sleep 5"])
        .spawn()
        .unwrap();
    let error = stop_limit::<()>(&mut child, "bounded").unwrap_err();
    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
    assert!(child.try_wait().unwrap().is_some());

    let mut child = Command::new("/bin/sh")
        .args(["-c", "sleep 5"])
        .spawn()
        .unwrap();
    let error = stop_with::<()>(&mut child, RuntimeError::new("failed")).unwrap_err();
    assert_eq!(error.kind, RuntimeErrorKind::General);
    assert!(child.try_wait().unwrap().is_some());
}

fn valid_limits(root: Option<&Path>) -> ProcessLimits<'_> {
    ProcessLimits {
        deadline: Instant::now() + Duration::from_secs(15),
        max_stdout_bytes: 64,
        max_stderr_bytes: 64,
        output_root: root,
        working_directory: None,
        max_output_bytes: 64,
    }
}
