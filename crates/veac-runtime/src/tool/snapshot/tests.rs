use std::time::{Duration, Instant};

use veac_artifact::read_verified_source_bounded_while;

use super::*;
use crate::RuntimeErrorKind;

#[test]
fn active_reports_deadline_without_losing_the_resource_limit_kind() {
    active(Instant::now() + Duration::from_secs(1)).unwrap();
    assert_eq!(
        active(Instant::now()).unwrap_err().kind,
        RuntimeErrorKind::ResourceLimit
    );
}

#[test]
fn artifact_errors_keep_resource_limits_distinct_from_io_failures() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::write(&source, b"payload").unwrap();
    let limited = read_verified_source_bounded_while(&source, None, 1, || true).unwrap_err();
    assert_eq!(
        artifact("read snapshot", limited).kind,
        RuntimeErrorKind::ResourceLimit
    );

    let missing = read_verified_source_bounded_while(
        &temp.path().join("missing"),
        None,
        MAX_IN_MEMORY_ARTIFACT_BYTES,
        || true,
    )
    .unwrap_err();
    assert_eq!(
        artifact("read snapshot", missing).kind,
        RuntimeErrorKind::General
    );
}

#[cfg(unix)]
#[test]
fn launch_rejects_a_missing_pinned_executable_with_exact_context() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("tool.sh");
    std::fs::write(&source, b"#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o755)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let pinned = PinnedExecutable::capture_until(&source, deadline).unwrap();
    std::fs::set_permissions(
        pinned.directory.path(),
        std::fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    std::fs::remove_file(&pinned.path).unwrap();

    let error = pinned.launch_until(deadline).unwrap_err();
    assert!(error.message.contains("launch pinned executable"));
}
