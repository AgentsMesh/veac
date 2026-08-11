use super::*;

#[test]
fn snapshot_deadlines_remain_resource_limits() {
    let error = tool_error(
        Path::new("ffprobe"),
        crate::RuntimeError::resource_limit("expired"),
    );
    assert!(matches!(
        error,
        ProbeError::ResourceLimit {
            operation: "tool snapshot"
        }
    ));
}
