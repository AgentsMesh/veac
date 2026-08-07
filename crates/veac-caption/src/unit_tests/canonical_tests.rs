use crate::{test_support::envelope, *};

#[test]
fn canonical_json_and_hash_are_stable() {
    let value = envelope();
    let first = canonical_caption_json(&value).unwrap();
    let decoded = decode_caption_json(&first).unwrap();
    assert_eq!(decoded, value);
    assert_eq!(first, canonical_caption_json(&decoded).unwrap());
    assert_eq!(
        caption_hash(&value).unwrap(),
        caption_hash(&decoded).unwrap()
    );
    assert_eq!(caption_hash(&value).unwrap().len(), 64);
    assert_eq!(canonical_caption_bytes(&value).unwrap(), first.as_bytes());
    assert!(first.contains(r#""native":null"#));
    assert!(!first.contains(r#""settings""#));
}

#[test]
fn schema_describes_the_versioned_envelope() {
    let schema = caption_json_schema().unwrap();
    let serialized = schema.to_string();
    assert!(serialized.contains("CaptionEnvelope"));
    assert!(serialized.contains("schema_version"));
    assert!(caption_track_insertion_json_schema()
        .unwrap()
        .to_string()
        .contains("CaptionTrackInsertionBindings"));
    assert!(caption_document_bindings_json_schema()
        .unwrap()
        .to_string()
        .contains("CaptionDocumentBindings"));
}

#[test]
fn loss_reports_are_canonical_and_duplicate_names_are_rejected() {
    let report = LossReport {
        losses: vec![CaptionLoss {
            cue_id: None,
            field: "styles".to_owned(),
            reason: "format has no styles".to_owned(),
        }],
    };
    assert_eq!(
        canonical_loss_report_json(&report).unwrap(),
        r#"{"losses":[{"cue_id":null,"field":"styles","reason":"format has no styles"}]}"#
    );
    let json = canonical_caption_json(&envelope()).unwrap();
    let duplicate = json.replacen(
        r#""schema_version":2"#,
        r#""schema_version":2,"schema_version":2"#,
        1,
    );
    assert!(matches!(
        decode_caption_json(&duplicate),
        Err(CaptionError::Json(_))
    ));
}

#[test]
fn canonical_entry_points_reject_invalid_documents_and_json() {
    let mut value = envelope();
    value.schema = "wrong".to_owned();
    assert!(matches!(
        canonical_caption_json(&value),
        Err(CaptionError::Validation(_))
    ));
    assert!(matches!(
        canonical_caption_bytes(&value),
        Err(CaptionError::Validation(_))
    ));
    assert!(matches!(
        caption_hash(&value),
        Err(CaptionError::Validation(_))
    ));
    assert!(matches!(
        decode_caption_json("{"),
        Err(CaptionError::Json(_))
    ));
}
