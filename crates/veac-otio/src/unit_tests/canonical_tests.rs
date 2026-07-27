use crate::{unit_tests::support::project, *};

#[test]
fn exported_otio_is_canonical_strict_json_with_public_schemas() {
    let result =
        export_sequence(&project(), &veac_ir::SequenceId::new("seq_main").unwrap()).unwrap();
    let json = canonical_otio_json(&result.timeline).unwrap();
    assert_eq!(
        canonical_otio_json(&decode_otio_json(&json).unwrap()).unwrap(),
        json
    );
    assert!(decode_otio_json(&json.replacen(
        "\"name\":\"Main\"",
        "\"name\":\"Main\",\"name\":\"Again\"",
        1
    ))
    .is_err());
    assert!(otio_loss_json_schema().unwrap().is_object());
    assert!(otio_import_bindings_json_schema().unwrap().is_object());
    assert!(otio_edit_proposal_json_schema().unwrap().is_object());
    assert_eq!(
        serde_json::from_str::<OtioLossReport>(
            &canonical_otio_loss_json(&result.loss_report).unwrap()
        )
        .unwrap(),
        result.loss_report
    );
}

#[test]
fn wrong_otio_headers_and_missing_sequences_fail_closed() {
    let project = project();
    assert!(export_sequence(&project, &veac_ir::SequenceId::new("seq_missing").unwrap()).is_err());
    let mut invalid = project.clone();
    invalid.project.timebase = 0;
    assert!(export_sequence(&invalid, &invalid.project.entry_sequence_id).is_err());
    let mut timeline = export_sequence(&project, &project.project.entry_sequence_id)
        .unwrap()
        .timeline;
    timeline.schema = "Timeline.99".to_owned();
    assert!(canonical_otio_json(&timeline).is_err());
}

#[test]
fn standard_enabled_defaults_decode_and_unknown_references_fail_closed() {
    let project = project();
    let timeline = export_sequence(&project, &project.project.entry_sequence_id)
        .unwrap()
        .timeline;
    let json = canonical_otio_json(&timeline).unwrap();
    let without_enabled = json
        .replace("\"enabled\":true,", "")
        .replace(",\"enabled\":true", "");
    assert!(!without_enabled.contains("\"enabled\":true"));
    let decoded = decode_otio_json(&without_enabled).unwrap();
    assert!(decoded.tracks.enabled);
    assert!(decoded.tracks.children[0].enabled);
    assert!(matches!(
        decoded.tracks.children[0].children[0],
        OtioItem::Gap { enabled: true, .. }
    ));

    let unknown = json.replacen("\"ExternalReference.1\"", "\"UnknownReference.9\"", 1);
    assert!(decode_otio_json(&unknown).is_err());
    let mut malformed = timeline;
    malformed.global_start_time = Some(OtioRationalTime {
        schema: OTIO_RATIONAL_TIME_SCHEMA.to_owned(),
        value: 0.0,
        rate: 0.0,
    });
    assert!(canonical_otio_json(&malformed).is_err());
}
