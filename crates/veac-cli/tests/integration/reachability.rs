use super::support::*;

const SOURCE: &str = include_str!("../fixtures/reachability.veac");

#[test]
fn unused_missing_and_remote_materials_do_not_block_planning() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, SOURCE);
    let mut envelope = read_project(&project);
    envelope.project.render_configs.truncate(1);
    envelope.project.sequences.truncate(1);
    envelope.project.sequences[0].tracks.truncate(1);
    envelope.project.authorship = None;
    envelope.project.sequences[0].authorship = None;
    write_project(&project, &envelope);
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .success();

    let mut envelope = read_project(&project);
    envelope.project.materials[0].source = veac_ir::MaterialSource::Remote {
        uri: "https://example.test/offline.mp4".into(),
    };
    write_project(&project, &envelope);
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn solo_folding_ignores_suppressed_offline_tracks() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, SOURCE);
    let mut envelope = read_project(&project);
    envelope.project.render_configs.truncate(1);
    envelope.project.sequences.truncate(1);
    envelope.project.sequences[0].tracks[0].state.solo = true;
    envelope.project.authorship = None;
    write_project(&project, &envelope);
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .success();

    let mut envelope = read_project(&project);
    envelope.project.sequences[0].tracks[0].state.solo = false;
    write_project(&project, &envelope);
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PATH_UNAVAILABLE"));
}

#[test]
fn selected_output_hydrates_only_its_reachable_sequence_graph() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, SOURCE);
    let mut envelope = read_project(&project);
    let main_sequence = envelope
        .project
        .render_configs
        .iter()
        .find(|output| output.deliverables[0].target.file_name() == Some("main.mp4"))
        .unwrap()
        .sequence_id
        .clone();
    envelope
        .project
        .sequences
        .iter_mut()
        .find(|sequence| sequence.id == main_sequence)
        .unwrap()
        .tracks[1]
        .state
        .enabled = false;
    write_project(&project, &envelope);
    let config = |target: &str| {
        envelope
            .project
            .render_configs
            .iter()
            .find(|output| output.deliverables[0].target.file_name() == Some(target))
            .unwrap()
            .id
            .to_string()
    };
    let main = config("main.mp4");
    let offline = config("offline.mp4");
    veac()
        .args(["plan", project.to_str().unwrap(), "--config", &main])
        .assert()
        .success();
    veac()
        .args(["plan", project.to_str().unwrap(), "--config", &offline])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PATH_UNAVAILABLE"));
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("RENDER_CONFIG_REQUIRED"));
}

fn read_project(path: &std::path::Path) -> veac_ir::ProjectEnvelope {
    veac_ir::decode_canonical_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn write_project(path: &std::path::Path, envelope: &veac_ir::ProjectEnvelope) {
    std::fs::write(path, veac_ir::canonical_json(envelope).unwrap()).unwrap();
}
