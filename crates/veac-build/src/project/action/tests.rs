#[test]
fn canonicalization_errors_keep_project_context() {
    let source = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let error = super::canonical_error(source);
    assert!(error.message().contains("project action"));
}
