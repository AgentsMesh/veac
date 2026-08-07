use super::*;

#[test]
fn manifest_schema_parse_errors_and_error_display_are_observable() {
    let schema = build_input_manifest_json_schema().unwrap();
    let encoded_schema = schema.to_string();
    assert!(encoded_schema.contains("schema_version"));
    assert!(encoded_schema.contains("\"const\":1"));
    assert!(encoded_schema.contains("\"enum\""));
    assert!(encoded_schema.contains("\"x-veac-max-utf8-bytes\":1048576"));

    let json = serde_json::to_string(&BuildInputManifestV1::empty()).unwrap();
    assert_eq!(
        parse_build_input_manifest(&json).unwrap().inputs,
        Vec::new()
    );

    let malformed = parse_build_input_manifest("{").unwrap_err();
    assert_eq!(malformed.code(), "PROGRAM_INPUT_MANIFEST_JSON");
    assert!(!malformed.message().is_empty());
    assert_eq!(
        malformed.to_string(),
        format!("{}: {}", malformed.code(), malformed.message())
    );
}

#[test]
fn manifest_runtime_shape_matches_name_and_utf8_byte_contracts() {
    let maximum = crate::program::expression::MAX_TEXT_VALUE_BYTES;
    let valid = "中".repeat(maximum / "中".len());
    let mut unicode = manifest(vec![binding(
        "title",
        BuildInputManifestValue::Text { value: valid },
    )]);
    assert!(unicode.validate_identity().is_ok());
    let BuildInputManifestValue::Text { value } = &mut unicode.inputs[0].value else {
        unreachable!()
    };
    value.push('中');
    assert_eq!(
        unicode.validate_identity().unwrap_err().code(),
        "PROGRAM_INPUT_VALUE"
    );

    let invalid_name = manifest(vec![binding(
        &"x".repeat(129),
        BuildInputManifestValue::Bool { value: true },
    )]);
    assert_eq!(
        invalid_name.validate_identity().unwrap_err().code(),
        "PROGRAM_INPUT_NAME"
    );
    let invalid_variant = manifest(vec![binding(
        "locale",
        BuildInputManifestValue::Enum {
            value: "zh/Hans".into(),
        },
    )]);
    assert_eq!(
        invalid_variant.validate_identity().unwrap_err().code(),
        "PROGRAM_INPUT_VALUE"
    );
}

#[test]
fn owned_and_borrowed_error_messages_preserve_their_contract() {
    let borrowed = BuildInputsError::new("BORROWED", std::hint::black_box("borrowed message"));
    assert_eq!(borrowed.code(), "BORROWED");
    assert_eq!(borrowed.message(), "borrowed message");

    let owned = BuildInputsError::new("OWNED", std::hint::black_box("owned message".to_owned()));
    assert_eq!(owned.code(), "OWNED");
    assert_eq!(owned.message(), "owned message");
    assert!(std::error::Error::source(&owned).is_none());
}

#[test]
fn manifest_identity_rejects_each_unsupported_component() {
    let mut wrong_schema = BuildInputManifestV1::empty();
    wrong_schema.schema = "https://invalid.example/build-inputs".to_owned();
    assert_eq!(
        wrong_schema.validate_identity().unwrap_err().code(),
        "PROGRAM_INPUT_MANIFEST_VERSION"
    );

    let mut wrong_version = BuildInputManifestV1::empty();
    wrong_version.schema_version += 1;
    assert_eq!(
        wrong_version.validate_identity().unwrap_err().code(),
        "PROGRAM_INPUT_MANIFEST_VERSION"
    );
}
