use super::relation_support::{relation_project, rename_import};
use super::support::project;
use crate::{export_sequence, import_veac_extension, propose_import};
use veac_ir::{apply_edit_batch, EditOutcome, OperationId, SequenceId};

#[test]
fn proposal_inserts_sequence_and_relations_atomically() {
    let source = relation_project();
    let timeline = export_sequence(&source, &SequenceId::new("seq_main").unwrap())
        .unwrap()
        .timeline;
    let mut imported = import_veac_extension(&timeline).unwrap();
    rename_import(&mut imported, "imported");
    let target = project();
    let proposal = propose_import(
        &target,
        &imported,
        OperationId::new("op_import_relations").unwrap(),
        false,
    )
    .unwrap();
    let result = match apply_edit_batch(&target, &proposal.batch) {
        EditOutcome::Applied { project, .. } => project,
        other => panic!("expected applied proposal, got {other:?}"),
    };
    assert_eq!(result.project.sequences[1], imported.sequence);
    assert_eq!(result.project.relations, imported.relations);
}

#[test]
fn proposal_rejects_relation_id_conflicts_without_mutating_target() {
    let source = relation_project();
    let timeline = export_sequence(&source, &SequenceId::new("seq_main").unwrap())
        .unwrap()
        .timeline;
    let mut imported = import_veac_extension(&timeline).unwrap();
    rename_import(&mut imported, "conflict");
    let target = relation_project();
    let before = target.clone();
    let error = propose_import(
        &target,
        &imported,
        OperationId::new("op_import_conflict").unwrap(),
        false,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("relation ID already exists"), "{error}");
    assert_eq!(target, before);
}

#[test]
fn proposal_revalidates_public_import_results_before_insertion() {
    let source = relation_project();
    let timeline = export_sequence(&source, &SequenceId::new("seq_main").unwrap())
        .unwrap()
        .timeline;
    let mut imported = import_veac_extension(&timeline).unwrap();
    rename_import(&mut imported, "tampered");
    imported.relations[0].sequence_id = SequenceId::new("seq_wrong").unwrap();
    let target = project();
    let before = target.clone();
    let error = propose_import(
        &target,
        &imported,
        OperationId::new("op_import_tampered").unwrap(),
        false,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("sequence relation graph is invalid"),
        "{error}"
    );
    assert_eq!(target, before);
}
