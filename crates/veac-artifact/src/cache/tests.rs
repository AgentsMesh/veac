use std::{fs, path::Path};

use super::*;
use crate::test_support::descriptor;

#[test]
fn cache_private_paths_fail_closed_without_writing_outside_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let store = ArtifactStore::new(&root);
    assert_eq!(store.root(), root);

    let descriptor = descriptor();
    let payload = b"payload";
    let record = ArtifactRecord {
        key: artifact_key(&descriptor).unwrap(),
        content: ContentDigest::sha256(payload),
        size_bytes: payload.len() as u64,
    };
    let outside = temp.path().join("outside/artifact");
    assert_eq!(
        io::read_while(store.root(), &outside, || true)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
    assert_eq!(
        write_atomic(store.root(), &outside, &descriptor, &record, payload)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
    assert_eq!(
        write_atomic(store.root(), Path::new("/"), &descriptor, &record, payload)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
    assert!(!temp.path().join("outside").exists());

    let file_root = temp.path().join("file-root");
    fs::write(&file_root, b"not a directory").unwrap();
    let error = write_atomic(
        &file_root,
        &file_root.join("artifact"),
        &descriptor,
        &record,
        payload,
    )
    .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::CorruptCache);
}

fn write_atomic(
    root: &Path,
    directory: &Path,
    descriptor: &ArtifactDescriptor,
    record: &ArtifactRecord,
    payload: &[u8],
) -> ArtifactResult<io::PublishOutcome> {
    io::write_atomic_while_with(
        root,
        directory,
        descriptor,
        record,
        payload,
        |_| {},
        |_| {},
        |_| {},
        || true,
    )
}

#[test]
fn same_key_publish_race_reuses_an_identical_winner_and_rejects_drift() {
    use std::cell::RefCell;

    let root = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(root.path());
    let expected = descriptor();
    let winner = RefCell::new(None);
    let loser = store
        .put_with(&expected, b"same", |_| {
            winner.replace(Some(store.put(&expected, b"same").unwrap()));
        })
        .unwrap();
    assert_eq!(winner.into_inner(), Some(loser));

    let other_root = tempfile::tempdir().unwrap();
    let other_store = ArtifactStore::new(other_root.path());
    let error = other_store
        .put_with(&expected, b"loser", |_| {
            other_store.put(&expected, b"winner").unwrap();
        })
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
    let key = artifact_key(&expected).unwrap();
    assert_eq!(other_store.get(&key).unwrap().unwrap().payload, b"winner");
}

#[test]
fn streamed_writer_rehashes_the_source_and_cleans_failed_staging() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let source = temp.path().join("source");
    fs::write(&source, b"payload").unwrap();
    let descriptor = descriptor();
    let key = artifact_key(&descriptor).unwrap();
    let directory = cache_directory(&root, &key);
    let record = ArtifactRecord {
        key,
        content: ContentDigest::sha256(b"different"),
        size_bytes: 9,
    };
    assert_eq!(
        io::write_file_atomic_while(&root, &directory, &descriptor, &record, &source, || true)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::IdentityMismatch
    );
    assert!(!directory.exists());
    assert!(fs::read_dir(directory.parent().unwrap())
        .unwrap()
        .all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".tmp-")));
}

#[cfg(unix)]
#[test]
fn streamed_writer_rejects_a_symlink_on_its_second_source_open() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let source = temp.path().join("source");
    let linked = temp.path().join("linked");
    fs::write(&source, b"payload").unwrap();
    symlink(&source, &linked).unwrap();
    let descriptor = descriptor();
    let key = artifact_key(&descriptor).unwrap();
    let record = ArtifactRecord {
        key: key.clone(),
        content: ContentDigest::sha256(b"payload"),
        size_bytes: 7,
    };
    assert_eq!(
        io::write_file_atomic_while(
            &root,
            &cache_directory(&root, &key),
            &descriptor,
            &record,
            &linked,
            || true,
        )
        .unwrap_err()
        .kind,
        ArtifactErrorKind::UnsafePath
    );
}

#[test]
fn streamed_writer_rejects_a_target_without_a_parent() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    fs::write(&source, b"payload").unwrap();
    let descriptor = descriptor();
    let record = ArtifactRecord {
        key: artifact_key(&descriptor).unwrap(),
        content: ContentDigest::sha256(b"payload"),
        size_bytes: 7,
    };
    assert_eq!(
        io::write_file_atomic_while(
            temp.path(),
            Path::new("/"),
            &descriptor,
            &record,
            &source,
            || true,
        )
        .unwrap_err()
        .kind,
        ArtifactErrorKind::UnsafePath
    );
}

fn cache_directory(root: &Path, key: &ContentDigest) -> std::path::PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}
