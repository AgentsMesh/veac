use tempfile::tempdir;
use veac_ir::{MaterialKind, MaterialSource, StreamChoice, StreamIntent};

use super::support::{
    canonical_project, identity, FakeEnvironment, GENERATED_SOURCE, MEDIA_SOURCE,
};

#[test]
fn canonical_load_reports_json_and_semantic_errors() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    assert_eq!(
        crate::canonical::load(&project)
            .unwrap()
            .project
            .id
            .to_string(),
        "prj_cli-test"
    );

    std::fs::write(&project, "{bad").unwrap();
    assert!(crate::canonical::load(&project)
        .unwrap_err()
        .to_string()
        .contains("CANONICAL_JSON"));

    let valid = canonical_project(&temp, GENERATED_SOURCE);
    let text = std::fs::read_to_string(&valid).unwrap().replacen(
        "\"entry_sequence_id\":\"seq_main\"",
        "\"entry_sequence_id\":\"seq_absent\"",
        1,
    );
    std::fs::write(&valid, text).unwrap();
    assert!(crate::canonical::load(&valid)
        .unwrap_err()
        .to_string()
        .contains("ENTRY_SEQUENCE_NOT_FOUND"));
}

#[test]
fn local_media_is_hydrated_without_persisting_machine_paths() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), "fixture").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let environment = FakeEnvironment::success();
    let hydrated = hydrate_all(&project, &environment).unwrap();
    let material = &hydrated.envelope.project.materials[0];
    assert_eq!(material.identity.as_ref(), Some(&environment.observed));
    assert!(material.probe.is_some());
    assert_eq!(
        hydrated.material_paths[&material.id],
        std::fs::canonicalize(temp.path().join("clip.mp4")).unwrap()
    );
    assert!(!std::fs::read_to_string(project)
        .unwrap()
        .contains(&temp.path().display().to_string()));
}

#[test]
fn authored_global_stream_intent_reaches_the_probe_contract() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), "fixture").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    envelope.project.materials[0].stream_intent.video =
        StreamChoice::GlobalIndex { global_index: 0 };
    write_envelope(&project, &envelope);
    let hydrated = hydrate_all(&project, &FakeEnvironment::success()).unwrap();
    assert_eq!(
        hydrated.envelope.project.materials[0]
            .probe
            .as_ref()
            .unwrap()
            .selected_video_stream
            .unwrap()
            .global_index,
        0
    );
}

#[test]
fn hydration_rejects_remote_missing_and_failed_materials() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), "fixture").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    envelope.project.materials[0].source = MaterialSource::Remote {
        uri: "https://example.test/clip.mp4".into(),
    };
    write_envelope(&project, &envelope);
    assert!(hydrate_all(&project, &FakeEnvironment::success())
        .unwrap_err()
        .to_string()
        .contains("REMOTE_MATERIAL_UNRESOLVED"));

    let missing = canonical_project(&temp, MEDIA_SOURCE);
    std::fs::remove_file(temp.path().join("clip.mp4")).unwrap();
    assert!(hydrate_all(&missing, &FakeEnvironment::success())
        .unwrap_err()
        .to_string()
        .contains("PATH_UNAVAILABLE"));

    std::fs::write(temp.path().join("clip.mp4"), "fixture").unwrap();
    let failed = FakeEnvironment {
        fail_probe: true,
        ..FakeEnvironment::success()
    };
    assert!(hydrate_all(&missing, &failed)
        .unwrap_err()
        .to_string()
        .contains("FAKE_PROBE"));
}

#[test]
fn hydration_enforces_identity_pins_and_canonical_hash_support() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), "fixture").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    envelope.project.materials[0].identity = Some(identity(0x22));
    write_envelope(&project, &envelope);
    assert!(hydrate_all(&project, &FakeEnvironment::success())
        .unwrap_err()
        .to_string()
        .contains("MATERIAL_IDENTITY_MISMATCH"));

    let unsupported = std::fs::read_to_string(&project).unwrap().replacen(
        "\"algorithm\":\"sha256\"",
        "\"algorithm\":\"blake3\"",
        1,
    );
    std::fs::write(&project, unsupported).unwrap();
    assert!(crate::canonical::load(&project)
        .unwrap_err()
        .to_string()
        .contains("MATERIAL_IDENTITY"));
}

#[test]
fn font_materials_are_hashed_without_ffprobe() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), "font bytes").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let material = &mut envelope.project.materials[0];
    material.kind = MaterialKind::Font;
    material.stream_intent = StreamIntent {
        video: StreamChoice::Disabled,
        audio: StreamChoice::Disabled,
    };
    envelope.project.sequences[0].tracks[0].clips.clear();
    write_envelope(&project, &envelope);
    let hydrated = hydrate_all(&project, &FakeEnvironment::success()).unwrap();
    assert!(hydrated.envelope.project.materials[0].probe.is_none());
}

fn write_envelope(path: &std::path::Path, envelope: &veac_ir::ProjectEnvelope) {
    std::fs::write(path, veac_ir::canonical_json(envelope).unwrap()).unwrap();
}

fn hydrate_all(
    path: &std::path::Path,
    environment: &dyn crate::environment::Environment,
) -> crate::CliResult<crate::canonical::HydratedProject> {
    let loaded = crate::canonical::load_local(path)?;
    let required = loaded
        .envelope
        .project
        .materials
        .iter()
        .map(|material| material.id.clone())
        .collect();
    crate::canonical::hydrate(loaded, &required, environment)
}
