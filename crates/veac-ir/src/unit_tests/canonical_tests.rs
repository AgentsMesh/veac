use std::error::Error;

use crate::{test_support::sample_project, *};

#[test]
fn canonical_json_is_jcs_deterministic_and_round_trips() {
    let project = sample_project();
    let first = canonical_json(&project).unwrap();
    let second = String::from_utf8(canonical_bytes(&project).unwrap()).unwrap();
    assert_eq!(first, second);
    assert!(!first.contains('\n'));
    assert!(first.starts_with("{\"executable\":"));
    assert_eq!(decode_canonical_json(&first).unwrap(), project);
}

#[test]
fn semantic_hash_excludes_bookkeeping_and_probe_facts() {
    let project = sample_project();
    let semantic = semantic_hash(&project).unwrap();
    let snapshot = snapshot_hash(&project).unwrap();
    assert_eq!(semantic.len(), 64);
    assert_eq!(snapshot.len(), 64);
    assert_ne!(semantic, snapshot);

    let mut observed = project.clone();
    observed.project.revision += 1;
    observed.project.applied_operations.push(AppliedOperation {
        id: OperationId::new("op_seen").unwrap(),
        request_hash: "0".repeat(64),
    });
    observed.project.materials[0]
        .probe
        .as_mut()
        .unwrap()
        .streams[0]
        .video
        .as_mut()
        .unwrap()
        .width = 640;
    assert_eq!(semantic_hash(&observed).unwrap(), semantic);
    assert_ne!(snapshot_hash(&observed).unwrap(), snapshot);

    observed.project.sequences[0].name = "Changed".to_owned();
    assert_ne!(semantic_hash(&observed).unwrap(), semantic);

    let mut styled = project.clone();
    if let ClipSource::Caption { style, .. } =
        &mut styled.project.sequences[0].tracks[1].clips[0].source
    {
        style.tracking_pixels = 1.0;
    }
    assert_ne!(semantic_hash(&styled).unwrap(), semantic);

    let mut invalid_history = project.clone();
    invalid_history
        .project
        .applied_operations
        .push(AppliedOperation {
            id: OperationId::new("op_invalid_hash").unwrap(),
            request_hash: "not-a-digest".to_owned(),
        });
    assert!(matches!(
        semantic_hash(&invalid_history),
        Err(CanonicalError::Validation(_))
    ));
}

#[test]
fn decode_rejects_malformed_unknown_and_semantically_invalid_json() {
    let malformed = decode_canonical_json("{").unwrap_err();
    assert!(matches!(malformed, CanonicalError::Json(_)));
    assert!(malformed.to_string().contains("canonical JSON error"));
    assert!(malformed.source().is_some());

    let mut value = serde_json::to_value(sample_project()).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("unknown".to_owned(), serde_json::Value::Bool(true));
    let unknown = decode_canonical_json(&value.to_string()).unwrap_err();
    assert!(matches!(unknown, CanonicalError::Json(_)));

    let mut invalid = sample_project();
    invalid.project.timebase = 0;
    let json = serde_json::to_string(&invalid).unwrap();
    let semantic = decode_canonical_json(&json).unwrap_err();
    assert!(matches!(semantic, CanonicalError::Validation(_)));
    assert!(semantic
        .to_string()
        .contains("canonical IR validation failed"));
    assert!(semantic.source().is_some());
    assert!(matches!(
        canonical_json(&invalid),
        Err(CanonicalError::Validation(_))
    ));
    assert!(matches!(
        canonical_bytes(&invalid),
        Err(CanonicalError::Validation(_))
    ));
}

#[test]
fn decoder_rejects_duplicate_names_and_unsafe_integers() {
    assert!(reject_duplicate_json_keys(r#"{"value":1}"#).is_ok());
    assert!(reject_duplicate_json_keys(r#"{"value":1,"value":2}"#).is_err());
    let canonical = canonical_json(&sample_project()).unwrap();
    let duplicate = canonical.replacen(
        "\"schema\":\"https://veac.dev/schemas/project\"",
        concat!(
            "\"schema\":\"https://veac.dev/schemas/project\",",
            "\"schema\":\"https://veac.dev/schemas/project\""
        ),
        1,
    );
    let error = decode_canonical_json(&duplicate).unwrap_err();
    assert!(error.to_string().contains("duplicate object name"));

    let mut value = serde_json::to_value(sample_project()).unwrap();
    value["project"]["revision"] = serde_json::Value::from(MAX_SAFE_INTEGER + 1);
    assert!(matches!(
        decode_canonical_json(&value.to_string()),
        Err(CanonicalError::Validation(_))
    ));

    let negative_revision = canonical.replacen("\"revision\":7", "\"revision\":-1", 1);
    assert!(matches!(
        decode_canonical_json(&negative_revision),
        Err(CanonicalError::Json(_))
    ));

    value["project"]["revision"] = serde_json::Value::from(MAX_SAFE_INTEGER);
    let safe = decode_canonical_json(&value.to_string()).unwrap();
    assert_eq!(
        decode_canonical_json(&canonical_json(&safe).unwrap()).unwrap(),
        safe
    );
}

#[test]
fn canonical_error_conversion_and_schema_are_public_contracts() {
    let json_error = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let error = CanonicalError::from(json_error);
    assert!(matches!(error, CanonicalError::Json(_)));

    let schema = project_json_schema().unwrap();
    let schema_text = serde_json::to_string(&schema).unwrap();
    assert!(schema_text.contains("ProjectEnvelope"));
    assert!(schema_text.contains("schema_version"));
    assert!(schema_text.contains("ClipSource"));
    assert!(schema_text.contains("TextLayout"));
    assert!(schema_text.contains("TextAnimation"));
    assert!(schema_text.contains("VectorShape"));
    assert!(schema_text.contains("GradientStop"));
    assert!(schema_text.contains("replaceable"));
    assert!(schema_text.contains("template_editable_text"));
    assert!(schema_text.contains("min_source_duration"));
    let required = schema["$defs"]["Clip"]["required"].as_array().unwrap();
    assert!(required.iter().any(|value| value == "replaceable"));
    assert!(required
        .iter()
        .any(|value| value == "template_editable_text"));
}

#[test]
fn canonical_clips_require_explicit_template_state() {
    let json = serde_json::to_value(sample_project()).unwrap();
    let mut missing = json;
    missing["project"]["sequences"][0]["tracks"][0]["clips"][0]
        .as_object_mut()
        .unwrap()
        .remove("replaceable");
    assert!(decode_canonical_json(&missing.to_string()).is_err());
}
