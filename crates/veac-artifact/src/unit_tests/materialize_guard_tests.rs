use std::cell::Cell;
use std::fs;
use std::path::Path;

use crate::{test_support, *};

#[test]
fn materialize_guard_checks_chunks_and_cleans_mid_copy_cancellation() {
    let temp = tempfile::tempdir().unwrap();
    let payload = vec![0x6d; 1024 * 1024 + 13];
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, &payload).unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();

    let success = temp.path().join("success.bin");
    let calls = Cell::new(0_usize);
    assert_eq!(
        materialize_while(&artifact, &success, || count(&calls, None)).unwrap(),
        fs::canonicalize(&success).unwrap()
    );
    assert_eq!(fs::read(&success).unwrap(), payload);
    assert!(calls.get() > 128);

    let cancelled = temp.path().join("cancelled.bin");
    let current = Cell::new(0_usize);
    let error = materialize_while(&artifact, &cancelled, || {
        count(&current, Some(calls.get() / 3))
    })
    .unwrap_err();
    assert_resource(error);
    assert!(!cancelled.exists());
    assert!(stage_directories(temp.path()).is_empty());
}

#[test]
fn materialize_guard_cleans_a_stage_cancelled_before_publish() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, b"payload").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    let destination = temp.path().join("cancelled.bin");

    let error = materialize_while(&artifact, &destination, || {
        stage_directories(temp.path()).is_empty()
    })
    .unwrap_err();
    assert_resource(error);
    assert!(!destination.exists());
    assert!(stage_directories(temp.path()).is_empty());
}

#[test]
fn materialize_guard_marks_post_publish_cancellation_as_committed() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, b"payload").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    let destination = temp.path().join("published.bin");

    let error = materialize_while(&artifact, &destination, || !destination.exists()).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert_eq!(error.commit_state, ArtifactCommitState::Committed);
    assert_eq!(fs::read(destination).unwrap(), b"payload");
    assert!(stage_directories(temp.path()).is_empty());
}

#[test]
fn existing_materialization_verification_uses_the_guard_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let payload = vec![0x41; 512 * 1024 + 7];
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, &payload).unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    let destination = temp.path().join("existing.bin");
    materialize(&artifact, &destination).unwrap();

    let calls = Cell::new(0_usize);
    materialize_while(&artifact, &destination, || count(&calls, None)).unwrap();
    let current = Cell::new(0_usize);
    let error = materialize_while(&artifact, &destination, || {
        count(&current, Some(calls.get() / 2))
    })
    .unwrap_err();
    assert_resource(error);
    assert_eq!(fs::read(destination).unwrap(), payload);
}

#[test]
fn expired_materialization_guard_fails_before_destination_access() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, b"payload").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    let destination = temp.path().join("output.bin");

    assert_resource(materialize_while(&artifact, &destination, || false).unwrap_err());
    assert!(!destination.exists());
}

fn count(calls: &Cell<usize>, limit: Option<usize>) -> bool {
    let next = calls.get() + 1;
    calls.set(next);
    limit.is_none_or(|limit| next < limit)
}

fn assert_resource(error: ArtifactError) {
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert_eq!(error.commit_state, ArtifactCommitState::NotCommitted);
}

fn stage_directories(parent: &Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(parent)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with(".veac-stage-"))
        })
        .collect()
}
