use veac_artifact::{verify_source_bounded_while, ContentDigest};

use super::*;

#[test]
fn staging_helpers_cover_names_guards_and_typed_errors() {
    let key = ContentDigest::sha256(b"payload");
    assert_eq!(file_name(&key), format!("{}.payload", key.value));
    assert!(active(&mut || true).is_ok());
    assert_eq!(
        active(&mut || false).unwrap_err().kind,
        WorkflowErrorKind::ResourceLimit
    );
    assert_eq!(
        unsafe_staging::<()>("unsafe").unwrap_err().kind,
        WorkflowErrorKind::UnsafeStaging
    );
    assert_eq!(
        unsafe_error("unsafe").kind,
        WorkflowErrorKind::UnsafeStaging
    );
    assert_eq!(
        resource::<()>("bounded").unwrap_err().kind,
        WorkflowErrorKind::ResourceLimit
    );
    assert_eq!(
        resource_error("bounded").kind,
        WorkflowErrorKind::ResourceLimit
    );
}

#[test]
fn payload_errors_keep_limits_distinct_from_unsafe_paths() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    std::fs::write(&payload, b"payload").unwrap();
    let limited = verify_source_bounded_while(&payload, None, 1, || true).unwrap_err();
    assert_eq!(
        payload_error(limited).kind,
        WorkflowErrorKind::ResourceLimit
    );
    let missing = verify_source_bounded_while(
        &temp.path().join("missing"),
        None,
        veac_artifact::MAX_IN_MEMORY_ARTIFACT_BYTES,
        || true,
    )
    .unwrap_err();
    assert_eq!(
        payload_error(missing).kind,
        WorkflowErrorKind::UnsafeStaging
    );
}
