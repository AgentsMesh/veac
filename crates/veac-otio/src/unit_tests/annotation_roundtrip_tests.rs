use crate::{unit_tests::annotation_support::*, *};

#[test]
fn extension_round_trips_typed_annotations_exactly() {
    let project = annotated_project();
    let exported = export_sequence(&project, &project.project.entry_sequence_id).unwrap();
    let loss = exported
        .loss_report
        .losses
        .iter()
        .find(|loss| loss.field == "annotations")
        .unwrap();
    assert!(loss.preserved_in_extension);
    assert_eq!(exported.timeline.metadata["veac"]["version"], 3);
    let imported = import_veac_extension(&exported.timeline).unwrap();
    assert_eq!(imported.annotations, project.project.annotations);
    assert!(imported.loss_report.is_empty());
}

#[test]
fn extension_rejects_bad_annotation_version_order_and_targets() {
    let project = annotated_project();
    let timeline = export_sequence(&project, &project.project.entry_sequence_id)
        .unwrap()
        .timeline;
    let mut version = timeline.clone();
    version.metadata.get_mut("veac").unwrap()["version"] = 1.into();
    assert!(import_veac_extension(&version)
        .unwrap_err()
        .to_string()
        .contains("version"));

    let mut duplicate = timeline.clone();
    let annotations = duplicate.metadata.get_mut("veac").unwrap()["annotations"]
        .as_array_mut()
        .unwrap();
    annotations.push(annotations[0].clone());
    assert!(import_veac_extension(&duplicate).is_err());

    let mut target = timeline;
    target.metadata.get_mut("veac").unwrap()["annotations"][0]["target"] = serde_json::json!({
        "type": "sequence",
        "sequence_id": "seq_absent"
    });
    assert!(import_veac_extension(&target)
        .unwrap_err()
        .to_string()
        .contains("outside"));
}

#[test]
fn export_omits_annotations_outside_the_selected_sequence_with_loss() {
    let mut project = annotated_project();
    let mut other = project.project.sequences[0].clone();
    other.id = veac_ir::SequenceId::new("seq_other").unwrap();
    other.tracks[0].id = veac_ir::TrackId::new("trk_other").unwrap();
    other.tracks[0].clips[0].id = veac_ir::ItemId::new("itm_other").unwrap();
    project.project.sequences.push(other);
    project.project.annotations.push(marker(
        "ann_other_sequence",
        veac_ir::AnnotationTarget::Sequence {
            sequence_id: veac_ir::SequenceId::new("seq_other").unwrap(),
        },
        veac_ir::AnnotationSpan::Untimed,
    ));
    project
        .project
        .annotations
        .sort_by(|left, right| left.id.cmp(&right.id));
    veac_ir::validate(&project).unwrap();

    let exported = export_sequence(&project, &project.project.entry_sequence_id).unwrap();
    let imported = import_veac_extension(&exported.timeline).unwrap();
    assert!(!imported
        .annotations
        .iter()
        .any(|value| value.id.as_str() == "ann_other_sequence"));
    assert!(exported.loss_report.losses.iter().any(|loss| {
        loss.pointer.ends_with("ann_other_sequence") && !loss.preserved_in_extension
    }));
}
