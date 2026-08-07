use crate::*;

use super::support::{progress_program, scalar};

#[test]
fn temporal_ids_are_closed_and_typed() {
    let id = TemporalProgramId::new("tpg_title-card").unwrap();
    assert_eq!(id.as_str(), "tpg_title-card");
    assert_eq!(id.to_string(), "tpg_title-card");
    let error = TemporalProgramId::new("program").unwrap_err();
    assert_eq!(error.expected_prefix(), "tpg_");
    assert_eq!(error.value(), "program");
    assert!(error.to_string().contains("tpg_"));
    assert!(TemporalBindingId::new("tbd_a").is_ok());
    assert!(TemporalParameterId::new("tpm_a").is_ok());
    assert!(TemporalProvenanceId::new("tpv_a").is_ok());
    assert!(TemporalSourceId::new("src_a").is_ok());
    assert!(TemporalDefinitionId::new("def_a").is_ok());
    assert!(TemporalLogicalKey::new("key_a").is_ok());
    assert_eq!(TemporalNodeId::new(7).get(), 7);
    assert_eq!(TemporalInputId::new(8).get(), 8);
}

#[test]
fn temporal_values_report_closed_types_and_sizes() {
    let values = [
        (
            TemporalValue::Boolean { value: true },
            TemporalType::Boolean,
            1,
        ),
        (
            TemporalValue::Integer { value: 4 },
            TemporalType::Integer,
            8,
        ),
        (scalar(1.0), TemporalType::Scalar, 8),
        (
            TemporalValue::Angle { degrees: 90.0 },
            TemporalType::Angle,
            8,
        ),
        (
            TemporalValue::Color {
                value: Color {
                    red: 1,
                    green: 2,
                    blue: 3,
                    alpha: 4,
                },
            },
            TemporalType::Color,
            4,
        ),
        (
            TemporalValue::Point {
                value: Point {
                    x: Length {
                        value: 1.0,
                        unit: LengthUnit::Pixels,
                    },
                    y: Length {
                        value: 2.0,
                        unit: LengthUnit::Pixels,
                    },
                },
            },
            TemporalType::Point,
            32,
        ),
        (
            TemporalValue::Rect {
                value: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                },
            },
            TemporalType::Rect,
            32,
        ),
        (
            TemporalValue::Text {
                value: "abc".to_owned(),
            },
            TemporalType::Text,
            27,
        ),
    ];
    for (value, expected_type, expected_bytes) in values {
        assert_eq!(value.value_type(), expected_type);
        assert_eq!(value.logical_bytes(), expected_bytes);
    }
    assert_eq!(TemporalClock::SequenceTime.value_type(), TemporalType::Time);
    assert_eq!(TemporalClock::ClipTime.value_type(), TemporalType::Time);
    assert_eq!(TemporalClock::SourceTime.value_type(), TemporalType::Time);
    assert_eq!(TemporalClock::Frame.value_type(), TemporalType::Integer);
    assert_eq!(TemporalClock::Progress.value_type(), TemporalType::Scalar);
}

#[test]
fn temporal_serde_is_tagged_and_rejects_unknown_fields() {
    let value = TemporalNodeKind::Literal { value: scalar(2.0) };
    assert_eq!(
        serde_json::to_value(&value).unwrap(),
        serde_json::json!({"type":"literal","value":{"type":"scalar","value":2.0}})
    );
    assert!(
        serde_json::from_value::<TemporalNodeKind>(serde_json::json!({
            "type":"literal", "value":{"type":"scalar","value":2.0}, "expression":"t*2"
        }))
        .is_err()
    );
    let schema = schemars::schema_for!(TemporalProgramLibrary);
    let schema = serde_json::to_value(schema).unwrap().to_string();
    assert!(schema.contains("TemporalNodeKind"));
    assert!(!schema.contains("expression"));
}

#[test]
fn content_digest_excludes_identity_and_provenance_but_tracks_semantics() {
    let program = progress_program();
    let digest = temporal_program_digest(&program).unwrap();
    assert_eq!(digest.len(), 64);
    assert!(digest
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));

    let mut moved = program.clone();
    moved.id = TemporalProgramId::new("tpg_other").unwrap();
    moved.provenance_id = TemporalProvenanceId::new("tpv_other").unwrap();
    moved.nodes[0].provenance_id = Some(TemporalProvenanceId::new("tpv_node").unwrap());
    assert_eq!(temporal_program_digest(&moved).unwrap(), digest);

    let mut changed = program;
    let TemporalNodeKind::Literal { value } = &mut changed.nodes[1].kind else {
        unreachable!()
    };
    *value = scalar(3.0);
    assert_ne!(temporal_program_digest(&changed).unwrap(), digest);
}

#[test]
fn temporal_errors_expose_stable_diagnostics() {
    let error = TemporalEvaluationError::new("CODE", "/node", "failed");
    assert_eq!(error.code(), "CODE");
    assert_eq!(error.pointer(), "/node");
    assert_eq!(error.message(), "failed");
    assert_eq!(error.to_string(), "CODE at /node: failed");
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn digest_serialization_errors_remain_typed() {
    let mut program = progress_program();
    let TemporalNodeKind::Literal { value } = &mut program.nodes[1].kind else {
        unreachable!()
    };
    *value = TemporalValue::Scalar { value: f64::NAN };
    let error = temporal_program_digest(&program).unwrap_err();
    assert!(error.to_string().contains("non-canonical number"));
    assert!(std::error::Error::source(&error).is_none());
    let json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let error = TemporalDigestError::Json(json);
    assert!(error.to_string().contains("canonicalization failed"));
    assert!(std::error::Error::source(&error).is_some());
}
