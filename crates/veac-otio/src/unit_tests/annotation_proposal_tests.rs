use crate::{
    unit_tests::{annotation_support::*, import_error_tests::standard, support::project},
    *,
};

#[test]
fn proposal_inserts_typed_annotations_after_their_owning_graph() {
    let source = annotated_project();
    let timeline = export_sequence(&source, &source.project.entry_sequence_id)
        .unwrap()
        .timeline;
    let mut imported = import_veac_extension(&timeline).unwrap();
    remap_imported(&mut imported);
    let target = project();
    let proposal = propose_import(
        &target,
        &imported,
        veac_ir::OperationId::new("op_otio_annotations").unwrap(),
        false,
    )
    .unwrap();
    assert!(matches!(
        proposal.batch.operations[0],
        veac_ir::EditOperation::EditStructure {
            edit: veac_ir::StructureEdit::InsertSequence { .. }
        }
    ));
    assert!(proposal.batch.operations[1..]
        .iter()
        .all(|operation| matches!(operation, veac_ir::EditOperation::InsertAnnotation { .. })));
    let veac_ir::EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&target, &proposal.batch)
    else {
        unreachable!()
    };
    assert_eq!(applied.project.annotations, imported.annotations);

    let mut invalid = proposal.batch;
    let veac_ir::EditOperation::InsertAnnotation { annotation } =
        invalid.operations.last_mut().unwrap()
    else {
        unreachable!()
    };
    annotation.target = veac_ir::AnnotationTarget::Sequence {
        sequence_id: veac_ir::SequenceId::new("seq_absent").unwrap(),
    };
    assert!(matches!(
        veac_ir::apply_edit_batch(&target, &invalid),
        veac_ir::EditOutcome::Rejected { .. }
    ));
    assert_eq!(target.project.sequences.len(), 1);
    assert!(target.project.annotations.is_empty());
}

#[test]
fn third_party_bound_import_explicitly_reports_annotation_loss() {
    let (timeline, bindings) = standard();
    assert!(!timeline.metadata.contains_key("veac"));
    let imported = import_bound_timeline(&timeline, &bindings).unwrap();
    assert!(imported.annotations.is_empty());
    assert!(imported.loss_report.losses.iter().any(|loss| {
        loss.field == "annotations"
            && !loss.preserved_in_extension
            && loss.reason.contains("no VEAC extension")
    }));
}

#[test]
fn proposal_rejects_conflicting_annotation_ids() {
    let source = annotated_project();
    let timeline = export_sequence(&source, &source.project.entry_sequence_id)
        .unwrap()
        .timeline;
    let mut imported = import_veac_extension(&timeline).unwrap();
    remap_imported(&mut imported);
    let mut target = project();
    let mut conflict = imported
        .annotations
        .iter()
        .find(|value| matches!(value.target, veac_ir::AnnotationTarget::Project))
        .unwrap()
        .clone();
    conflict.payload = veac_ir::AnnotationPayload::Marker {
        label: "conflict".into(),
        color: None,
    };
    target.project.annotations.push(conflict);
    let error = propose_import(
        &target,
        &imported,
        veac_ir::OperationId::new("op_otio_annotation_conflict").unwrap(),
        false,
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("annotation ann_project conflicts"));
}
