use crate::arguments::OtioCommand;
use crate::unit_tests::support::{canonical_project, GENERATED_SOURCE};

mod bound;
mod errors;

#[test]
fn otio_export_requires_loss_acknowledgement_and_writes_both_contracts() {
    let temp = tempfile::tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let timeline = temp.path().join("timeline.otio");
    let loss = temp.path().join("timeline.loss.json");
    assert!(crate::commands::otio(OtioCommand::Export {
        project: project.clone(),
        sequence: None,
        output: Some(timeline.clone()),
        loss_report: None,
        allow_lossy: false,
    })
    .is_err());
    crate::commands::otio(OtioCommand::Export {
        project,
        sequence: None,
        output: Some(timeline.clone()),
        loss_report: Some(loss.clone()),
        allow_lossy: true,
    })
    .unwrap();
    veac_otio::decode_otio_json(&std::fs::read_to_string(timeline).unwrap()).unwrap();
    let report: veac_otio::OtioLossReport =
        serde_json::from_str(&std::fs::read_to_string(loss).unwrap()).unwrap();
    assert!(!report.is_empty());

    let stdout_loss = temp.path().join("stdout.loss.json");
    crate::commands::otio(OtioCommand::Export {
        project: canonical_project(&temp, GENERATED_SOURCE),
        sequence: None,
        output: None,
        loss_report: Some(stdout_loss.clone()),
        allow_lossy: true,
    })
    .unwrap();
    assert!(stdout_loss.is_file());
}

#[test]
fn otio_extension_proposal_is_atomic_and_applicable() {
    let temp = tempfile::tempdir().unwrap();
    let source = canonical_project(&temp, GENERATED_SOURCE);
    let timeline = temp.path().join("timeline.otio");
    let loss = temp.path().join("timeline.loss.json");
    crate::commands::otio(OtioCommand::Export {
        project: source.clone(),
        sequence: None,
        output: Some(timeline.clone()),
        loss_report: Some(loss),
        allow_lossy: true,
    })
    .unwrap();
    let target = target_project(&temp, &source);
    let output = temp.path().join("proposal.json");
    crate::commands::otio(OtioCommand::Propose {
        project: target.clone(),
        timeline,
        bindings: None,
        operation_id: "op_otio_cli".to_owned(),
        output: Some(output.clone()),
        loss_report: None,
        allow_lossy: false,
    })
    .unwrap();
    let proposal: veac_otio::OtioEditProposal =
        serde_json::from_str(&std::fs::read_to_string(output).unwrap()).unwrap();
    let target = crate::canonical::load(&target).unwrap();
    assert!(proposal.batch.atomic);
    assert!(matches!(
        veac_ir::apply_edit_batch(&target, &proposal.batch),
        veac_ir::EditOutcome::Applied { .. }
    ));
}

fn target_project(temp: &tempfile::TempDir, source: &std::path::Path) -> std::path::PathBuf {
    let mut target = crate::canonical::load(source).unwrap();
    let sequence = &mut target.project.sequences[0];
    sequence.id = veac_ir::SequenceId::new("seq_target").unwrap();
    sequence.tracks[0].id = veac_ir::TrackId::new("trk_target").unwrap();
    sequence.tracks[0].clips[0].id = veac_ir::ItemId::new("itm_target").unwrap();
    target.project.id = veac_ir::ProjectId::new("prj_target").unwrap();
    target.project.entry_sequence_id = sequence.id.clone();
    target.project.render_configs[0].sequence_id = sequence.id.clone();
    sequence.authorship = None;
    sequence.tracks[0].clips[0].authorship = None;
    target.project.authorship = None;
    let path = temp.path().join("target.json");
    std::fs::write(&path, veac_ir::canonical_json(&target).unwrap()).unwrap();
    path
}
