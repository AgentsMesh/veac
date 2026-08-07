use std::fs;

use veac_ir::{HashAlgorithm, MaterialKind, MaterialSource};
use veac_lang::program::apply_executable_source_edit_path;

use super::support::{batch, Fixture};

#[test]
fn audio_path_and_digest_edit_publish_one_consistent_preview() {
    let fixture = Fixture::new();
    let before = fs::read(&fixture.module).unwrap();
    let body = r#"{ audio_resource(identifier("voice"), resource_file("assets/new.wav"),
      sha256("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"),
      stream_auto()) }"#;
    let preview =
        apply_executable_source_edit_path(&fixture.entry, &batch(&fixture.entry, "voice", body))
            .unwrap();
    let material = preview
        .built
        .envelope()
        .project
        .materials
        .iter()
        .find(|material| material.kind == MaterialKind::Audio)
        .unwrap();
    assert_eq!(
        material.source,
        MaterialSource::File {
            uri: "assets/new.wav".to_owned()
        }
    );
    let identity = material.identity.as_ref().unwrap();
    assert_eq!(identity.algorithm, HashAlgorithm::Sha256);
    assert_eq!(identity.digest, "d".repeat(64));
    assert!(preview.source().unwrap().contains("assets/new.wav"));
    assert_eq!(fs::read(&fixture.module).unwrap(), before);
}
