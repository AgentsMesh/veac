#[test]
fn canonicalization_errors_keep_project_context() {
    let source = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let error = super::canonical_error(source);
    assert!(error.message().contains("project action"));
}

#[test]
fn source_graph_action_schema_separates_authored_and_complete_identity() {
    let revision = super::ProjectSourceGraphRevision {
        root_module: "main.veac".to_owned(),
        authored_source_graph_sha256: "a".repeat(64),
        complete_source_graph_sha256: "b".repeat(64),
        authored_module_count: 1,
        authored_modules: vec!["main.veac".to_owned()],
    };
    assert_eq!(
        serde_json::to_value(revision).unwrap(),
        serde_json::json!({
            "root_module": "main.veac",
            "authored_source_graph_sha256": "a".repeat(64),
            "complete_source_graph_sha256": "b".repeat(64),
            "authored_module_count": 1,
            "authored_modules": ["main.veac"],
        })
    );
    assert_eq!(super::PROJECT_ACTION_VERSION, 6);
}
