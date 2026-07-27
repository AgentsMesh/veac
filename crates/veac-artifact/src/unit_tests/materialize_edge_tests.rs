use std::{fs, path::Path};

use crate::{materialize, test_support, ArtifactErrorKind, ArtifactStore, VerifiedArtifact};

#[test]
fn materialize_rejects_an_existing_same_size_content_mismatch() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = artifact(temp.path(), b"abc");
    let destination = temp.path().join("existing.bin");
    fs::write(&destination, b"xyz").unwrap();
    assert_eq!(
        materialize(&artifact, &destination).unwrap_err().kind,
        ArtifactErrorKind::IdentityMismatch
    );
    assert_eq!(fs::read(destination).unwrap(), b"xyz");
}

#[test]
fn materialize_rejects_a_destination_without_a_file_name() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = artifact(temp.path(), b"payload");
    assert_eq!(
        materialize(&artifact, Path::new("/")).unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
}

#[test]
fn materialize_rejects_a_payload_that_stopped_being_a_regular_file() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = artifact(temp.path(), b"payload");
    fs::remove_file(artifact.payload_path()).unwrap();
    fs::create_dir(artifact.payload_path()).unwrap();
    let destination = temp.path().join("output.bin");
    assert_eq!(
        materialize(&artifact, &destination).unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
    assert!(!destination.exists());
}

#[test]
fn materialize_ignores_a_predictable_existing_temporary_file() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = artifact(temp.path(), b"payload");
    let destination = temp.path().join("output.bin");
    let predictable = temp.path().join(format!(
        ".output.bin.veac-materialize-{}-0",
        std::process::id()
    ));
    fs::write(&predictable, b"sentinel").unwrap();

    materialize(&artifact, &destination).unwrap();

    assert_eq!(fs::read(destination).unwrap(), b"payload");
    assert_eq!(fs::read(predictable).unwrap(), b"sentinel");
}

fn artifact(root: &Path, bytes: &[u8]) -> VerifiedArtifact {
    let store = ArtifactStore::new(root.join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, bytes).unwrap();
    store.open(&record.key).unwrap().unwrap()
}
