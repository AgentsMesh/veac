use super::{empty_entry, error_code};

#[test]
fn local_instances_require_local_ids_and_unique_names() {
    let plain = r#"component sequence shell {
  instance sequence child from shell {}
  body {}
}"#;
    assert_eq!(error_code(&empty_entry(plain)), "PROGRAM_EXPECTED_LOCAL_ID");

    let duplicate = r#"component sequence shell {
  instance sequence @child from leaf {}
  instance sequence @child from leaf {}
  body {}
}
component sequence leaf { body {} }"#;
    assert_eq!(
        error_code(&empty_entry(duplicate)),
        "PROGRAM_DUPLICATE_LOCAL_INSTANCE"
    );
}

#[test]
fn unused_components_resolve_every_nested_component_reference() {
    let source = r#"component sequence shell {
  instance sequence @child from missing {}
  body {}
}"#;
    assert_eq!(
        error_code(&empty_entry(source)),
        "PROGRAM_COMPONENT_NOT_FOUND"
    );
}
