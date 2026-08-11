use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

use super::{material_kind, materials, verify_authored_identity};
use crate::commands::project::execution::backend::inputs::AuthorizedMaterial;
use veac_ir::{
    HashAlgorithm, Material, MaterialId, MaterialKind, MaterialSource, MediaIdentity, StreamChoice,
    StreamIntent,
};
use veac_lang::program::MaterialInputKind;

fn envelope() -> veac_ir::ProjectEnvelope {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../veac-ir/tests/fixtures/minimal-project.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn material(kind: MaterialKind, uri: &str, digest: Option<&str>) -> Material {
    Material {
        id: MaterialId::new("med_test").unwrap(),
        kind,
        source: MaterialSource::File {
            uri: uri.to_owned(),
        },
        identity: digest.map(|digest| MediaIdentity {
            algorithm: HashAlgorithm::Sha256,
            digest: digest.to_owned(),
        }),
        stream_intent: StreamIntent {
            video: StreamChoice::Auto,
            audio: StreamChoice::Auto,
        },
        probe: None,
        authorship: None,
    }
}

fn binding(kind: MaterialInputKind, digest: &str, path: &std::path::Path) -> AuthorizedMaterial {
    AuthorizedMaterial {
        kind,
        sha256: digest.to_owned(),
        path: path.to_owned(),
    }
}

fn required() -> BTreeSet<MaterialId> {
    BTreeSet::from([MaterialId::new("med_test").unwrap()])
}

#[test]
fn material_kinds_cover_the_closed_build_input_domain() {
    let cases = [
        (MaterialInputKind::Video, MaterialKind::Video),
        (MaterialInputKind::Audio, MaterialKind::Audio),
        (MaterialInputKind::Image, MaterialKind::Image),
        (MaterialInputKind::Font, MaterialKind::Font),
        (MaterialInputKind::Lut1d, MaterialKind::Lut1d),
        (MaterialInputKind::Lut3d, MaterialKind::Lut3d),
    ];
    for (input, expected) in cases {
        assert_eq!(material_kind(input), expected);
    }
}

#[test]
fn authored_identity_must_exist_and_match_the_authority() {
    let path = std::path::Path::new("font.ttf");
    let authority = binding(MaterialInputKind::Font, &"1".repeat(64), path);
    assert!(
        verify_authored_identity(&material(MaterialKind::Font, "font.ttf", None), &authority)
            .is_err()
    );

    let mut wrong = material(MaterialKind::Font, "font.ttf", Some(&"2".repeat(64)));
    assert!(verify_authored_identity(&wrong, &authority).is_err());
    wrong.identity.as_mut().unwrap().digest = "1".repeat(64);
    assert!(verify_authored_identity(&wrong, &authority).is_ok());
    wrong.identity.as_mut().unwrap().algorithm = HashAlgorithm::Blake3;
    assert!(verify_authored_identity(&wrong, &authority).is_err());
}

#[test]
fn hydration_rejects_missing_remote_and_incompatible_authorities() {
    let mut value = envelope();
    value.project.materials = vec![material(
        MaterialKind::Video,
        "clip.mp4",
        Some(&"1".repeat(64)),
    )];
    let required = required();
    assert!(materials(&mut value, &required, &BTreeMap::new()).is_err());

    value.project.materials[0].source = MaterialSource::Remote {
        uri: "https://example.invalid/clip.mp4".to_owned(),
    };
    assert!(materials(&mut value, &required, &BTreeMap::new()).is_err());

    value.project.materials[0].source = MaterialSource::File {
        uri: "clip.mp4".to_owned(),
    };
    let authority = BTreeMap::from([(
        "clip.mp4".to_owned(),
        binding(
            MaterialInputKind::Audio,
            &"1".repeat(64),
            std::path::Path::new("missing.mp4"),
        ),
    )]);
    assert!(materials(&mut value, &required, &authority).is_err());

    value.project.materials.clear();
    assert!(materials(&mut value, &required, &BTreeMap::new()).is_err());
}

#[test]
fn font_hydration_hashes_bytes_and_rejects_tampering() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(b"test-font-bytes").unwrap();
    let identity = veac_runtime::asset::sha256_identity(file.path()).unwrap();
    let mut value = envelope();
    value.project.materials = vec![material(
        MaterialKind::Font,
        "font.ttf",
        Some(&identity.digest),
    )];
    let mut authority = BTreeMap::from([(
        "font.ttf".to_owned(),
        binding(MaterialInputKind::Font, &identity.digest, file.path()),
    )]);
    let paths = materials(&mut value, &required(), &authority).unwrap();
    assert_eq!(paths[&MaterialId::new("med_test").unwrap()], file.path());

    authority.get_mut("font.ttf").unwrap().sha256 = "0".repeat(64);
    value.project.materials[0].identity.as_mut().unwrap().digest = "0".repeat(64);
    assert!(materials(&mut value, &required(), &authority).is_err());
}
