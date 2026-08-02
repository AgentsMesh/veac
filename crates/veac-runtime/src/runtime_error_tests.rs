use super::*;

#[test]
fn runtime_error_preserves_its_message() {
    let error = RuntimeError::new("render failed");
    assert_eq!(error.kind, RuntimeErrorKind::General);
    assert_eq!(error.message, "render failed");
    assert_eq!(error.to_string(), "render failed");
    let as_error: &dyn std::error::Error = &error;
    assert!(as_error.source().is_none());
}
