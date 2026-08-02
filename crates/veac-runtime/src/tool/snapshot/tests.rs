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
