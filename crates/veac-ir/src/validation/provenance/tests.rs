use super::*;

#[test]
fn aggregate_authorship_budget_is_fail_closed() {
    let mut validator = Validator {
        authorship_bytes: MAX_AUTHORSHIP_BYTES + 1,
        ..Validator::default()
    };
    validator.entity_authorship(
        &EntityAuthorship {
            logical_path: vec![LogicalPathSegment::new("root")],
            events: Vec::new(),
        },
        "/authorship",
        "root",
    );
    assert!(validator
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "AUTHORSHIP_BUDGET"));
}
