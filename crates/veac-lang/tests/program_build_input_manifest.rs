use veac_lang::program::{
    build_input_manifest_json_schema, parse_build_input_manifest, BuildInputManifestValue,
    MAX_BUILD_INPUTS, MAX_BUILD_INPUT_MANIFEST_BYTES,
};

const INTEGER_MANIFEST: &str = r#"{
  "schema":"https://veac.dev/schemas/build-inputs",
  "schema_version":1,
  "inputs":[{"name":"count","value":{"type":"int","value":7}}]
}"#;

#[test]
fn manifest_is_versioned_typed_and_deny_unknown() {
    let parsed = parse_build_input_manifest(INTEGER_MANIFEST).unwrap();
    assert!(matches!(
        parsed.inputs[0].value,
        BuildInputManifestValue::Integer { value: 7 }
    ));

    let unknown = INTEGER_MANIFEST.replacen("\"inputs\"", "\"unknown\":true,\"inputs\"", 1);
    assert_eq!(
        parse_build_input_manifest(&unknown).unwrap_err().code(),
        "PROGRAM_INPUT_MANIFEST_JSON"
    );
}

#[test]
fn manifest_input_count_is_bounded() {
    let mut oversized = parse_build_input_manifest(INTEGER_MANIFEST).unwrap();
    oversized
        .inputs
        .resize(MAX_BUILD_INPUTS + 1, oversized.inputs[0].clone());
    assert_eq!(
        oversized.validate_identity().unwrap_err().code(),
        "PROGRAM_INPUT_LIMIT"
    );
}

#[test]
fn schema_publishes_the_language_spelling_and_runtime_limit() {
    let schema = build_input_manifest_json_schema().unwrap().to_string();
    assert!(schema.contains("\"int\""));
    assert!(schema.contains("\"enum\""));
    assert!(schema.contains(&format!("\"maxItems\":{MAX_BUILD_INPUTS}")));
}

#[test]
fn parser_rejects_oversized_manifests_before_deserialization() {
    let oversized = " ".repeat(MAX_BUILD_INPUT_MANIFEST_BYTES + 1);
    assert_eq!(
        parse_build_input_manifest(&oversized).unwrap_err().code(),
        "PROGRAM_INPUT_MANIFEST_LIMIT"
    );
}
