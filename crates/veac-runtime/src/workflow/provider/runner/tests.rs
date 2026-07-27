use veac_artifact::read_verified_source_bounded_while;

use super::*;

#[test]
fn typed_error_mappers_keep_resource_limits_distinct() {
    assert_eq!(
        tool_failure(crate::RuntimeError::resource_limit("deadline")).kind,
        WorkflowErrorKind::ResourceLimit
    );
    assert_eq!(
        tool_failure(crate::RuntimeError::new("spawn")).kind,
        WorkflowErrorKind::ToolFailure
    );
    assert_eq!(
        workflow_tool_error("late", crate::RuntimeError::resource_limit("deadline")).kind,
        WorkflowErrorKind::ResourceLimit
    );
    assert_eq!(
        workflow_tool_error("io", crate::RuntimeError::new("io")).kind,
        WorkflowErrorKind::ToolFailure
    );
    assert_eq!(
        protocol::<()>("protocol").unwrap_err().kind,
        WorkflowErrorKind::ProtocolViolation
    );
    assert_eq!(
        resource::<()>("bounded").unwrap_err().kind,
        WorkflowErrorKind::ResourceLimit
    );
}

#[test]
fn artifact_error_mapper_handles_limit_and_io_failures() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    std::fs::write(&payload, b"payload").unwrap();
    let limited = read_verified_source_bounded_while(&payload, None, 1, || true).unwrap_err();
    assert_eq!(
        artifact_failure(limited).kind,
        WorkflowErrorKind::ResourceLimit
    );
    let missing = read_verified_source_bounded_while(
        &temp.path().join("missing"),
        None,
        veac_artifact::MAX_IN_MEMORY_ARTIFACT_BYTES,
        || true,
    )
    .unwrap_err();
    assert_eq!(artifact_failure(missing).kind, WorkflowErrorKind::Artifact);
}
