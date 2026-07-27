use crate::arguments::OtioCommand;
use crate::unit_tests::support::{canonical_project, MEDIA_SOURCE};

#[test]
fn bound_otio_requires_acknowledgement_and_emits_loss_with_proposal() {
    let temp = tempfile::tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let envelope = crate::canonical::load(&project).unwrap();
    let mut timeline = veac_otio::export_sequence(&envelope, &envelope.project.entry_sequence_id)
        .unwrap()
        .timeline;
    timeline.metadata.clear();
    let timeline_path = temp.path().join("third-party.otio");
    std::fs::write(
        &timeline_path,
        veac_otio::canonical_otio_json(&timeline).unwrap(),
    )
    .unwrap();
    let bindings = bindings(&temp, &envelope);
    let proposal = temp.path().join("bound-proposal.json");
    let command = |allow_lossy, loss_report| OtioCommand::Propose {
        project: project.clone(),
        timeline: timeline_path.clone(),
        bindings: Some(bindings.clone()),
        operation_id: "op_bound_unit".to_owned(),
        output: Some(proposal.clone()),
        loss_report,
        allow_lossy,
    };
    assert!(crate::commands::otio(command(false, None)).is_err());
    let loss = temp.path().join("bound.loss.json");
    crate::commands::otio(command(true, Some(loss.clone()))).unwrap();
    assert!(proposal.is_file() && loss.is_file());
}

fn bindings(temp: &tempfile::TempDir, envelope: &veac_ir::ProjectEnvelope) -> std::path::PathBuf {
    let sequence = &envelope.project.sequences[0];
    let value = veac_otio::OtioImportBindings {
        timebase: envelope.project.timebase,
        sequence_id: veac_ir::SequenceId::new("seq_bound_unit").unwrap(),
        sequence_name: "Bound".to_owned(),
        settings: sequence.settings.clone(),
        tracks: vec![veac_otio::OtioTrackBinding {
            track_id: veac_ir::TrackId::new("trk_bound_unit").unwrap(),
            order: 1,
        }],
        clip_ids: std::collections::BTreeMap::from([(
            "/tracks/0/children/0".to_owned(),
            veac_ir::ItemId::new("itm_bound_unit").unwrap(),
        )]),
        media: vec![veac_otio::OtioMediaBinding {
            target_url: "clip.mp4".to_owned(),
            material: envelope.project.materials[0].clone(),
        }],
    };
    let path = temp.path().join("bindings.json");
    std::fs::write(&path, serde_json_canonicalizer::to_string(&value).unwrap()).unwrap();
    path
}
