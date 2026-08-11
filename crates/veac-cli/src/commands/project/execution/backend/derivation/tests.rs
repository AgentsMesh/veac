use super::workflow_error;
use veac_build::CancellationToken;

fn workflow_failure() -> veac_runtime::workflow::WorkflowError {
    std::io::Error::other("failure").into()
}

#[test]
fn workflow_failures_distinguish_tool_errors_from_cancellation() {
    let token = CancellationToken::new();
    let failed = workflow_error(workflow_failure(), &token);
    assert!(failed
        .to_string()
        .contains("project media derivation failed"));
    assert!(failed
        .to_string()
        .contains("workflow filesystem operation failed: failure"));

    token.cancel();
    let cancelled = workflow_error(workflow_failure(), &token);
    assert!(cancelled.to_string().contains("cancelled"));
    assert!(!cancelled.to_string().contains("failure"));
}
