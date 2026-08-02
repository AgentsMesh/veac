use crate::{
    artifact_descriptor_json_schema, build_manifest_json_schema,
    execution_binding_manifest_json_schema, media_artifact_request_json_schema,
    package_manifest_json_schema,
};

#[test]
fn public_contract_schemas_are_strict_deterministic_objects() {
    for (schema, required) in [
        (
            artifact_descriptor_json_schema().unwrap(),
            &["kind", "producer", "parameters"][..],
        ),
        (
            build_manifest_json_schema().unwrap(),
            &["plan_hash", "inputs", "tools"][..],
        ),
        (
            package_manifest_json_schema().unwrap(),
            &["plan_hash", "project_path", "project_content", "entries"][..],
        ),
        (
            execution_binding_manifest_json_schema().unwrap(),
            &["plan_hash", "inputs"][..],
        ),
        (
            media_artifact_request_json_schema().unwrap(),
            &["source_identity", "producer", "spec"][..],
        ),
    ] {
        assert!(schema["properties"].is_object());
        assert_eq!(schema["additionalProperties"], false);
        let names = schema["required"].as_array().unwrap();
        for name in required {
            assert!(names.iter().any(|value| value == name));
        }
        let first = serde_json::to_vec(&schema).unwrap();
        let second = serde_json::to_vec(&schema).unwrap();
        assert_eq!(first, second);
    }
}
