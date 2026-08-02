use super::support::*;

#[test]
fn otio_extension_round_trip_produces_an_editable_atomic_batch() {
    let temp = tempdir().unwrap();
    let source = compile_ir(&temp, GENERATED_SOURCE);
    let timeline = temp.path().join("timeline.otio");
    let loss = temp.path().join("timeline.loss.json");
    veac()
        .args([
            "otio",
            "export",
            source.to_str().unwrap(),
            "--output",
            timeline.to_str().unwrap(),
            "--allow-lossy",
            "--loss-report",
            loss.to_str().unwrap(),
        ])
        .assert()
        .success();
    let target = target_project(&temp, &source);
    let proposal = temp.path().join("proposal.json");
    veac()
        .args([
            "otio",
            "propose",
            target.to_str().unwrap(),
            timeline.to_str().unwrap(),
            "--operation-id",
            "op_otio_e2e",
            "--output",
            proposal.to_str().unwrap(),
        ])
        .assert()
        .success();
    let wrapper: veac_otio::OtioEditProposal =
        serde_json::from_str(&std::fs::read_to_string(&proposal).unwrap()).unwrap();
    let batch = temp.path().join("batch.json");
    std::fs::write(
        &batch,
        veac_ir::canonical_edit_batch_json(&wrapper.batch).unwrap(),
    )
    .unwrap();
    veac()
        .args(["edit", target.to_str().unwrap(), batch.to_str().unwrap()])
        .assert()
        .success();
    let edited = veac_ir::decode_canonical_json(&std::fs::read_to_string(target).unwrap()).unwrap();
    assert_eq!(edited.project.sequences.len(), 2);
}

#[test]
fn third_party_otio_requires_bindings_and_explicit_loss_acknowledgement() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let timeline = exported_standard(&temp, &project);
    let bindings = bindings(&temp, &project);
    let proposal = temp.path().join("bound-proposal.json");
    let base = [
        "otio",
        "propose",
        project.to_str().unwrap(),
        timeline.to_str().unwrap(),
        "--bindings",
        bindings.to_str().unwrap(),
        "--operation-id",
        "op_bound_otio",
        "--output",
        proposal.to_str().unwrap(),
    ];
    veac()
        .args(base)
        .assert()
        .failure()
        .stderr(predicate::str::contains("OTIO_LOSS_UNACKNOWLEDGED"));
    let loss = temp.path().join("bound.loss.json");
    veac()
        .args(base)
        .args(["--allow-lossy", "--loss-report", loss.to_str().unwrap()])
        .assert()
        .success();
    assert!(proposal.is_file() && loss.is_file());
}

fn exported_standard(temp: &TempDir, project: &std::path::Path) -> std::path::PathBuf {
    let envelope =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(project).unwrap()).unwrap();
    let mut timeline = veac_otio::export_sequence(&envelope, &envelope.project.entry_sequence_id)
        .unwrap()
        .timeline;
    timeline.metadata.clear();
    let path = temp.path().join("third-party.otio");
    std::fs::write(&path, veac_otio::canonical_otio_json(&timeline).unwrap()).unwrap();
    path
}

fn bindings(temp: &TempDir, project: &std::path::Path) -> std::path::PathBuf {
    let envelope =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(project).unwrap()).unwrap();
    let sequence = &envelope.project.sequences[0];
    let value = veac_otio::OtioImportBindings {
        timebase: envelope.project.timebase,
        sequence_id: veac_ir::SequenceId::new("seq_bound").unwrap(),
        sequence_name: "Bound".to_owned(),
        settings: sequence.settings.clone(),
        tracks: vec![veac_otio::OtioTrackBinding {
            track_id: veac_ir::TrackId::new("trk_bound").unwrap(),
            order: 1,
        }],
        clip_ids: std::collections::BTreeMap::from([(
            "/tracks/0/children/0".to_owned(),
            veac_ir::ItemId::new("itm_bound").unwrap(),
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

fn target_project(temp: &TempDir, source: &std::path::Path) -> std::path::PathBuf {
    let mut target =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(source).unwrap()).unwrap();
    let sequence = &mut target.project.sequences[0];
    sequence.id = veac_ir::SequenceId::new("seq_target").unwrap();
    sequence.tracks[0].id = veac_ir::TrackId::new("trk_target").unwrap();
    sequence.tracks[0].clips[0].id = veac_ir::ItemId::new("itm_target").unwrap();
    target.project.id = veac_ir::ProjectId::new("prj_target").unwrap();
    target.project.entry_sequence_id = sequence.id.clone();
    target.project.render_configs[0].sequence_id = sequence.id.clone();
    let path = temp.path().join("target.json");
    std::fs::write(&path, veac_ir::canonical_json(&target).unwrap()).unwrap();
    path
}
