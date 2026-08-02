use super::*;
use crate::{test_support, ArtifactCommitState};
use std::cell::Cell;

#[test]
fn buffered_guard_cleans_an_unsealed_final_directory() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let key = artifact_key(&descriptor).unwrap();
    let target = store.directory(&key);
    let error = store
        .put_while(&descriptor, b"payload", || !target.exists())
        .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert_eq!(error.commit_state, ArtifactCommitState::NotCommitted);
    assert!(store.get(&key).unwrap().is_none());
    assert!(!target.exists());
}

#[test]
fn buffered_guard_marks_a_post_publish_cancellation_as_committed() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let key = artifact_key(&descriptor).unwrap();
    let target = store.directory(&key);
    let error = store
        .put_while(&descriptor, b"payload", || !target.join("sealed").exists())
        .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert_eq!(error.commit_state, ArtifactCommitState::Committed);
    assert_eq!(store.get(&key).unwrap().unwrap().payload, b"payload");
}

#[test]
fn conflict_winner_reverification_consumes_the_same_guard() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let winner_exists = Cell::new(false);
    let error = store
        .put_while_with(
            &descriptor,
            b"payload",
            |_| {
                store.put(&descriptor, b"payload").unwrap();
                winner_exists.set(true);
            },
            || !winner_exists.get(),
        )
        .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert_eq!(error.commit_state, ArtifactCommitState::NotCommitted);
    let key = artifact_key(&descriptor).unwrap();
    assert_eq!(store.get(&key).unwrap().unwrap().payload, b"payload");
}
