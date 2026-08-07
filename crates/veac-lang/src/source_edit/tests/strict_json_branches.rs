use super::*;

#[test]
fn strict_json_walker_visits_empty_collections_and_every_scalar_kind() {
    for input in ["{}", "[]", "true", "0", "-1", "1.5", "null", r#""text""#] {
        assert!(matches!(
            decode_source_edit_batch_json(input),
            Err(SourceEditJsonError::Json(_))
        ));
    }
}

#[test]
fn strict_json_rejects_duplicates_nested_inside_arrays() {
    let input = r#"[{"name":"first","name":"second"}]"#;
    let error = decode_source_edit_batch_json(input)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("duplicate object name"),
        "unexpected error: {error}"
    );
}

#[test]
fn strict_json_requires_the_deserializer_to_reach_end_of_input() {
    assert!(matches!(
        decode_source_edit_batch_json("{} {}"),
        Err(SourceEditJsonError::Json(_))
    ));
}
