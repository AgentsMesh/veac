use crate::{unit_tests::support::project, *};

#[test]
fn proposal_is_revision_checked_reviewable_and_applies_atomically() {
    let project = project();
    let exported = export_sequence(&project, &project.project.entry_sequence_id).unwrap();
    let mut imported = import_veac_extension(&exported.timeline).unwrap();
    imported.sequence.id = veac_ir::SequenceId::new("seq_imported").unwrap();
    imported.sequence.tracks[0].id = veac_ir::TrackId::new("trk_imported").unwrap();
    imported.sequence.tracks[0].clips[0].id = veac_ir::ItemId::new("itm_imported").unwrap();
    let proposal = propose_import(
        &project,
        &imported,
        veac_ir::OperationId::new("op_otio_import").unwrap(),
        false,
    )
    .unwrap();
    assert_eq!(proposal.batch.base_revision, 3);
    assert!(proposal.batch.atomic);
    let veac_ir::EditOutcome::Applied {
        project: applied,
        new_revision,
        ..
    } = veac_ir::apply_edit_batch(&project, &proposal.batch)
    else {
        panic!("OTIO import proposal must apply atomically")
    };
    assert_eq!(new_revision, 4);
    assert!(applied
        .project
        .sequences
        .iter()
        .any(|sequence| sequence.id.as_str() == "seq_imported"));
    let json = canonical_otio_proposal_json(&proposal).unwrap();
    assert_eq!(
        serde_json::from_str::<OtioEditProposal>(&json).unwrap(),
        proposal
    );
}

#[test]
fn proposal_rejects_unacknowledged_loss_and_identity_conflicts() {
    let project = project();
    let exported = export_sequence(&project, &project.project.entry_sequence_id).unwrap();
    let mut imported = import_veac_extension(&exported.timeline).unwrap();
    imported.sequence.id = veac_ir::SequenceId::new("seq_other").unwrap();
    imported.loss_report.losses.push(OtioLoss {
        pointer: "/tracks/0".to_owned(),
        field: "effects".to_owned(),
        reason: "unsupported".to_owned(),
        preserved_in_extension: false,
    });
    assert!(matches!(
        propose_import(
            &project,
            &imported,
            veac_ir::OperationId::new("op_loss").unwrap(),
            false
        ),
        Err(OtioError::Loss(_))
    ));
    imported.materials[0].source = veac_ir::MaterialSource::File {
        uri: "other.mp4".to_owned(),
    };
    assert!(propose_import(
        &project,
        &imported,
        veac_ir::OperationId::new("op_conflict").unwrap(),
        true,
    )
    .is_err());
}

#[test]
fn proposal_inserts_multicam_groups_before_the_imported_sequence() {
    let mut source = project();
    source.project.multicam_groups.push(group());
    let exported = export_sequence(&source, &source.project.entry_sequence_id).unwrap();
    let mut imported = import_veac_extension(&exported.timeline).unwrap();
    imported.sequence.id = veac_ir::SequenceId::new("seq_multicam_import").unwrap();
    imported.sequence.tracks[0].id = veac_ir::TrackId::new("trk_multicam_import").unwrap();
    imported.sequence.tracks[0].clips[0].id = veac_ir::ItemId::new("itm_multicam_import").unwrap();
    let target = project();
    let proposal = propose_import(
        &target,
        &imported,
        veac_ir::OperationId::new("op_multicam_import").unwrap(),
        false,
    )
    .unwrap();
    assert!(matches!(
        proposal.batch.operations[0],
        veac_ir::EditOperation::EditStructure {
            edit: veac_ir::StructureEdit::InsertMulticamGroup { .. }
        }
    ));
    assert!(matches!(
        proposal.batch.operations[1],
        veac_ir::EditOperation::EditStructure {
            edit: veac_ir::StructureEdit::InsertSequence { .. }
        }
    ));
    assert!(matches!(
        veac_ir::apply_edit_batch(&target, &proposal.batch),
        veac_ir::EditOutcome::Applied { .. }
    ));
}

pub(super) fn group() -> veac_ir::MulticamGroup {
    veac_ir::MulticamGroup {
        id: veac_ir::MulticamGroupId::new("mcg_otio").unwrap(),
        sync: veac_ir::MulticamSync {
            basis: veac_ir::MulticamSyncBasis::Manual,
            reference_angle_id: veac_ir::MulticamAngleId::new("ang_a").unwrap(),
        },
        angles: ["ang_a", "ang_b"]
            .into_iter()
            .map(|id| veac_ir::MulticamAngle {
                id: veac_ir::MulticamAngleId::new(id).unwrap(),
                material_id: veac_ir::MaterialId::new("med_video").unwrap(),
                source_offset: crate::unit_tests::support::time(0),
            })
            .collect(),
    }
}
