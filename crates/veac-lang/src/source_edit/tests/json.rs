use super::*;

#[test]
fn production_decoder_round_trips_canonical_contract_json() {
    let expected = batch();
    let canonical = canonical_source_edit_batch_json(&expected).unwrap();
    assert_eq!(decode_source_edit_batch_json(&canonical).unwrap(), expected);
    assert_eq!(
        canonical_source_edit_batch_json(&expected).unwrap(),
        canonical
    );

    let schema = source_edit_batch_json_schema().unwrap();
    assert!(schema.to_string().contains("authored_source_graph_sha256"));
    assert!(schema.to_string().contains("complete_source_graph_sha256"));
}

#[test]
fn production_decoder_rejects_top_level_and_nested_duplicate_names() {
    let canonical = canonical_source_edit_batch_json(&batch()).unwrap();
    let top_level = canonical.replacen(r#"{"atomic":true,"#, r#"{"atomic":true,"atomic":true,"#, 1);
    assert!(decode_source_edit_batch_json(&top_level)
        .unwrap_err()
        .to_string()
        .contains("duplicate object name"));

    let digest = "a".repeat(64);
    let nested = canonical.replacen(
        &format!(r#""authored_source_graph_sha256":"{digest}""#),
        &format!(
            r#""authored_source_graph_sha256":"{digest}","authored_source_graph_sha256":"{digest}""#
        ),
        1,
    );
    assert!(decode_source_edit_batch_json(&nested)
        .unwrap_err()
        .to_string()
        .contains("duplicate object name"));
}

#[test]
fn production_decoder_rejects_unknown_fields_and_invalid_contracts() {
    let canonical = canonical_source_edit_batch_json(&batch()).unwrap();
    let unknown = canonical.replacen(
        r#"{"atomic":true,"#,
        r#"{"atomic":true,"unknown":false,"#,
        1,
    );
    assert!(matches!(
        decode_source_edit_batch_json(&unknown),
        Err(SourceEditJsonError::Json(_))
    ));

    let invalid = canonical.replacen(r#""atomic":true"#, r#""atomic":false"#, 1);
    assert!(matches!(
        decode_source_edit_batch_json(&invalid),
        Err(SourceEditJsonError::Invalid(
            SourceEditError::AtomicRequired
        ))
    ));
}

#[test]
fn strict_decoder_walks_every_json_primitive_before_type_rejection() {
    for value in ["-1", "1.5", "null"] {
        assert!(matches!(
            decode_source_edit_batch_json(value),
            Err(SourceEditJsonError::Json(_))
        ));
    }
}

#[test]
fn json_error_exposes_its_underlying_error() {
    let error = decode_source_edit_batch_json("{").unwrap_err();
    assert!(std::error::Error::source(&error).is_some());
    let invalid = canonical_source_edit_batch_json(&SourceEditBatch::new(
        veac_ir::OperationId::new("op_empty").unwrap(),
        revision('a'),
    ))
    .unwrap_err();
    assert!(invalid.to_string().contains("invalid source-edit batch"));
    assert!(std::error::Error::source(&invalid).is_some());
}

#[test]
fn production_decoder_rejects_oversized_json_before_parsing() {
    let input = " ".repeat(MAX_SOURCE_EDIT_JSON_BYTES + 1);
    assert!(matches!(
        decode_source_edit_batch_json(&input),
        Err(SourceEditJsonError::Invalid(
            SourceEditError::SourceEditJsonTooLarge { .. }
        ))
    ));
}
