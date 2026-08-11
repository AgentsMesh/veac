use super::*;

#[test]
fn inactive_guard_is_a_typed_cancellation() {
    let error = active(&mut || false).unwrap_err();
    assert_eq!(error.kind, super::super::WorkflowErrorKind::ResourceLimit);
    assert!(error.to_string().contains("cancelled"));
}
