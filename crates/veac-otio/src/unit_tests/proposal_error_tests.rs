use crate::{
    unit_tests::{proposal_tests::group, support::project},
    *,
};

fn imported() -> (veac_ir::ProjectEnvelope, OtioImportResult) {
    let project = project();
    let timeline = export_sequence(&project, &project.project.entry_sequence_id)
        .unwrap()
        .timeline;
    let imported = import_veac_extension(&timeline).unwrap();
    (project, imported)
}

fn rename(imported: &mut OtioImportResult, suffix: &str) {
    imported.sequence.id = veac_ir::SequenceId::new(format!("seq_{suffix}")).unwrap();
    imported.sequence.tracks[0].id = veac_ir::TrackId::new(format!("trk_{suffix}")).unwrap();
    imported.sequence.tracks[0].clips[0].id =
        veac_ir::ItemId::new(format!("itm_{suffix}")).unwrap();
}

#[test]
fn proposal_rejects_sequence_identity_and_invalid_imported_graphs() {
    let (project, mut imported) = imported();
    assert!(propose_import(
        &project,
        &imported,
        veac_ir::OperationId::new("op_same_sequence").unwrap(),
        false,
    )
    .is_err());
    imported.sequence.id = veac_ir::SequenceId::new("seq_duplicate_children").unwrap();
    let error = propose_import(
        &project,
        &imported,
        veac_ir::OperationId::new("op_duplicate_children").unwrap(),
        false,
    )
    .unwrap_err();
    assert!(error.to_string().contains("DUPLICATE_TRACK_ID"));
}

#[test]
fn proposal_inserts_new_materials_before_the_sequence() {
    let (project, mut imported) = imported();
    rename(&mut imported, "new_material");
    imported.materials[0].id = veac_ir::MaterialId::new("med_imported").unwrap();
    imported.sequence.tracks[0].clips[0].source = veac_ir::ClipSource::Media {
        material_id: imported.materials[0].id.clone(),
    };
    let proposal = propose_import(
        &project,
        &imported,
        veac_ir::OperationId::new("op_new_material").unwrap(),
        false,
    )
    .unwrap();
    assert!(matches!(
        proposal.batch.operations[0],
        veac_ir::EditOperation::EditStructure {
            edit: veac_ir::StructureEdit::InsertMaterial { .. }
        }
    ));
    assert!(matches!(
        veac_ir::apply_edit_batch(&project, &proposal.batch),
        veac_ir::EditOutcome::Applied { .. }
    ));
}

#[test]
fn proposal_reuses_equal_multicam_groups_and_rejects_conflicts() {
    let (mut project, mut imported) = imported();
    let value = group();
    project.project.multicam_groups.push(value.clone());
    imported.multicam_groups.push(value);
    rename(&mut imported, "existing_group");
    let proposal = propose_import(
        &project,
        &imported,
        veac_ir::OperationId::new("op_existing_group").unwrap(),
        false,
    )
    .unwrap();
    assert_eq!(proposal.batch.operations.len(), 1);

    project.project.multicam_groups[0].sync.basis = veac_ir::MulticamSyncBasis::Timecode;
    assert!(propose_import(
        &project,
        &imported,
        veac_ir::OperationId::new("op_group_conflict").unwrap(),
        false,
    )
    .is_err());
}
