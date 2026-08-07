use tempfile::{tempdir, TempDir};
use veac_ir::{Material, MaterialId, MaterialKind, MaterialSource, StreamChoice, StreamIntent};

use super::support::{
    canonical_project, identity, write_project, FakeEnvironment, GENERATED_SOURCE,
};

#[test]
fn project_material_probe_uses_authored_stream_intent_for_every_media_kind() {
    for kind in [
        MaterialKind::Video,
        MaterialKind::Audio,
        MaterialKind::Image,
    ] {
        let temp = tempdir().unwrap();
        let environment = FakeEnvironment::success();
        let project = project_with_material(
            &temp,
            kind,
            MaterialSource::File {
                uri: "clip.mp4".to_owned(),
            },
        );
        std::fs::write(temp.path().join("clip.mp4"), b"fixture").unwrap();
        crate::commands::probe(&project, Some("med_footage"), None, &environment).unwrap();
        assert_eq!(
            environment.probe_paths.borrow().as_slice(),
            &[std::fs::canonicalize(temp.path().join("clip.mp4")).unwrap()]
        );
        assert_eq!(
            environment.probe_intents.borrow().as_slice(),
            &[stream_intent(kind)]
        );
    }
}

#[test]
fn project_material_probe_rejects_invalid_missing_and_non_media_ids() {
    let temp = tempdir().unwrap();
    let plain = canonical_project(&temp, GENERATED_SOURCE);
    assert_code(&plain, "bad id", "MATERIAL_ID_INVALID");
    assert_code(&plain, "med_absent", "MATERIAL_NOT_FOUND");

    let font = project_with_material(
        &temp,
        MaterialKind::Font,
        MaterialSource::File {
            uri: "font.ttf".to_owned(),
        },
    );
    assert_code(&font, "med_footage", "MATERIAL_NOT_PROBEABLE");
}

#[test]
fn project_material_probe_rejects_remote_and_failed_probes() {
    let temp = tempdir().unwrap();
    let remote = project_with_material(
        &temp,
        MaterialKind::Video,
        MaterialSource::Remote {
            uri: "https://example.test/clip.mp4".to_owned(),
        },
    );
    assert_code(&remote, "med_footage", "REMOTE_MATERIAL_UNRESOLVED");

    let local = project_with_material(
        &temp,
        MaterialKind::Video,
        MaterialSource::File {
            uri: "clip.mp4".to_owned(),
        },
    );
    std::fs::write(temp.path().join("clip.mp4"), b"fixture").unwrap();
    let failed = FakeEnvironment {
        fail_probe: true,
        ..FakeEnvironment::success()
    };
    let error = crate::commands::probe(&local, Some("med_footage"), None, &failed).unwrap_err();
    assert!(error.to_string().contains("FAKE_PROBE"));
}

fn project_with_material(
    temp: &TempDir,
    kind: MaterialKind,
    source: MaterialSource,
) -> std::path::PathBuf {
    let project = canonical_project(temp, GENERATED_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let stream_intent = stream_intent(kind);
    envelope.project.materials.push(Material {
        id: MaterialId::new("med_footage").unwrap(),
        kind,
        source,
        identity: Some(identity(0x11)),
        stream_intent,
        probe: None,
        authorship: None,
    });
    write_project(&project, &envelope);
    project
}

fn stream_intent(kind: MaterialKind) -> StreamIntent {
    match kind {
        MaterialKind::Video | MaterialKind::Image => StreamIntent {
            video: StreamChoice::GlobalIndex { global_index: 0 },
            audio: StreamChoice::Disabled,
        },
        MaterialKind::Audio => StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::GlobalIndex { global_index: 1 },
        },
        MaterialKind::Font | MaterialKind::Lut1d | MaterialKind::Lut3d => StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
    }
}

fn assert_code(project: &std::path::Path, material: &str, code: &str) {
    let error = crate::commands::probe(project, Some(material), None, &FakeEnvironment::success())
        .unwrap_err();
    assert!(error.to_string().contains(code), "{error}");
}
