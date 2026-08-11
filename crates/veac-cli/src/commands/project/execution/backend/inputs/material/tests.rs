use super::*;

#[test]
fn file_extensions_map_to_the_closed_material_kinds() {
    for (name, expected) in [
        ("clip.mp4", MaterialInputKind::Video),
        ("sound.WAV", MaterialInputKind::Audio),
        ("still.png", MaterialInputKind::Image),
        ("font.otf", MaterialInputKind::Font),
        ("curve.lut", MaterialInputKind::Lut1d),
        ("grade.cube", MaterialInputKind::Lut3d),
    ] {
        assert_eq!(kind_from_path(Path::new(name)).unwrap(), expected);
    }
    assert!(kind_from_path(Path::new("unknown.bin")).is_err());
    assert!(kind_from_path(Path::new("extensionless")).is_err());
}

#[test]
fn produced_project_artifact_kinds_map_without_guessing() {
    assert_eq!(
        kind_from_artifact(ArtifactKind::VideoMaster).unwrap(),
        MaterialInputKind::Video
    );
    assert_eq!(
        kind_from_artifact(ArtifactKind::AudioFile).unwrap(),
        MaterialInputKind::Audio
    );
    assert_eq!(
        kind_from_artifact(ArtifactKind::StillImage).unwrap(),
        MaterialInputKind::Image
    );
    assert!(kind_from_artifact(ArtifactKind::Analysis).is_err());
}

#[test]
fn source_snapshot_verification_checks_content_and_size() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("source.bin");
    std::fs::write(&path, b"verified").unwrap();
    let mut snapshot = ProjectFileSnapshot {
        path: "source.bin".into(),
        content: ContentDigest::sha256(b"verified"),
        size_bytes: 8,
    };
    verify_snapshot(&path, &snapshot).unwrap();
    snapshot.size_bytes = 7;
    assert!(verify_snapshot(&path, &snapshot).is_err());
    snapshot.size_bytes = 8;
    snapshot.content = ContentDigest::sha256(b"different");
    assert!(verify_snapshot(&path, &snapshot).is_err());
}

#[test]
fn material_manifest_value_preserves_authority_and_stream_defaults() {
    let value = material_value(
        MaterialInputKind::Image,
        "stills/hero.png",
        &"a".repeat(64),
        MaterialInputAuthority::ProjectMaterial,
    );
    let BuildInputManifestValue::Material {
        kind,
        path,
        authority,
        video_stream,
        audio_stream,
        ..
    } = value
    else {
        panic!("expected material")
    };
    assert_eq!(kind, MaterialInputKind::Image);
    assert_eq!(path, "stills/hero.png");
    assert_eq!(authority, MaterialInputAuthority::ProjectMaterial);
    assert_eq!((video_stream, audio_stream), (None, None));
}
