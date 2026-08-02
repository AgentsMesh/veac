use std::fs;

use crate::{test_support, *};

#[test]
fn materialize_streams_verified_content_and_reuses_an_exact_file() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let bytes = vec![0x5a; 256 * 1024];
    let record = store.put(&descriptor, &bytes).unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    let destination = temp.path().join("materialized.bin");
    let first = materialize(&artifact, &destination).unwrap();
    assert_eq!(first, fs::canonicalize(&destination).unwrap());
    assert_eq!(fs::read(&destination).unwrap(), bytes);
    assert_eq!(materialize(&artifact, &destination).unwrap(), first);
    assert!(temp_entries(temp.path()).is_empty());
}

#[test]
fn materialize_rejects_different_existing_content_without_replacing_it() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = artifact(temp.path(), b"artifact");
    let destination = temp.path().join("existing.bin");
    fs::write(&destination, b"different").unwrap();
    assert_eq!(
        materialize(&artifact, &destination).unwrap_err().kind,
        ArtifactErrorKind::IdentityMismatch
    );
    assert_eq!(fs::read(destination).unwrap(), b"different");
}

#[test]
fn materialize_rejects_missing_parent_directory_and_parent_segments() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = artifact(temp.path(), b"artifact");
    assert_eq!(
        materialize(&artifact, &temp.path().join("missing/output.bin"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::Io
    );
    assert_eq!(
        materialize(&artifact, &temp.path().join("child/../output.bin"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
}

#[cfg(unix)]
#[test]
fn materialize_rejects_symlink_and_directory_destinations() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let artifact = artifact(temp.path(), b"artifact");
    let target = temp.path().join("target");
    fs::write(&target, b"target").unwrap();
    let linked = temp.path().join("linked");
    symlink(&target, &linked).unwrap();
    assert_eq!(
        materialize(&artifact, &linked).unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
    let directory = temp.path().join("directory");
    fs::create_dir(&directory).unwrap();
    assert_eq!(
        materialize(&artifact, &directory).unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
    assert_eq!(fs::read(target).unwrap(), b"target");
}

#[cfg(unix)]
#[test]
fn materialize_rejects_a_symlink_parent() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let artifact = artifact(temp.path(), b"artifact");
    let real = temp.path().join("real");
    fs::create_dir(&real).unwrap();
    let linked = temp.path().join("linked-parent");
    symlink(&real, &linked).unwrap();
    assert_eq!(
        materialize(&artifact, &linked.join("output.bin"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
    assert!(!real.join("output.bin").exists());
}

#[test]
fn materialize_revalidates_payload_and_cleans_its_temporary_file() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = artifact(temp.path(), b"artifact");
    fs::write(artifact.payload_path(), b"changed").unwrap();
    let destination = temp.path().join("output.bin");
    assert_eq!(
        materialize(&artifact, &destination).unwrap_err().kind,
        ArtifactErrorKind::IdentityMismatch
    );
    assert!(!destination.exists());
    assert!(temp_entries(temp.path()).is_empty());
}

fn artifact(root: &std::path::Path, bytes: &[u8]) -> VerifiedArtifact {
    let store = ArtifactStore::new(root.join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, bytes).unwrap();
    store.open(&record.key).unwrap().unwrap()
}

fn temp_entries(root: &std::path::Path) -> Vec<String> {
    fs::read_dir(root)
        .unwrap()
        .filter_map(|entry| {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            name.starts_with(".veac-stage-").then_some(name)
        })
        .collect()
}
