use std::cell::Cell;

use crate::{test_support, *};

#[test]
fn expired_guards_fail_before_any_cache_access() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let key = artifact_key(&descriptor).unwrap();

    assert_resource(store.get_while(&key, || false).unwrap_err());
    assert_resource(store.open_while(&key, || false).unwrap_err());
    assert_resource(
        store
            .open_verified_bounded_while(&key, &descriptor, 1, || false)
            .unwrap_err(),
    );
    assert_resource(
        store
            .put_while(&descriptor, b"payload", || false)
            .unwrap_err(),
    );
    assert_resource(store.remove_while(&key, || false).unwrap_err());
    assert!(!store.root().exists());
}

#[test]
fn guarded_memory_reads_check_each_payload_chunk_and_cancel_mid_read() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let payload = vec![0x5a; 2 * 1024 * 1024 + 17];
    let record = store.put(&descriptor, &payload).unwrap();

    let get_calls = Cell::new(0_usize);
    let cached = store
        .get_while(&record.key, || count(&get_calls, None))
        .unwrap()
        .unwrap();
    assert_eq!(cached.payload, payload);
    assert!(get_calls.get() > 128);
    let current = Cell::new(0_usize);
    let error = store
        .get_while(&record.key, || count(&current, Some(get_calls.get() / 2)))
        .unwrap_err();
    assert_resource(error);

    let open_calls = Cell::new(0_usize);
    assert!(store
        .open_while(&record.key, || count(&open_calls, None))
        .unwrap()
        .is_some());
    assert!(open_calls.get() > 96);
    let current = Cell::new(0_usize);
    let error = store
        .open_while(&record.key, || count(&current, Some(open_calls.get() / 2)))
        .unwrap_err();
    assert_resource(error);
    assert!(store.get(&record.key).unwrap().is_some());
}

#[test]
fn guarded_remove_cancels_before_commit_and_cleans_after_commit() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, b"payload").unwrap();
    let calls = Cell::new(0_usize);
    let error = store
        .remove_while(&record.key, || count(&calls, Some(4)))
        .unwrap_err();
    assert_resource(error);
    assert!(store.get(&record.key).unwrap().is_some());

    let target = cache_directory(store.root(), &record.key);
    let error = store
        .remove_while(&record.key, || target.exists())
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert_eq!(error.commit_state, ArtifactCommitState::Committed);
    assert!(store.get(&record.key).unwrap().is_none());
    assert!(std::fs::read_dir(target.parent().unwrap())
        .unwrap()
        .next()
        .is_none());

    let record = store.put(&descriptor, b"payload").unwrap();
    let calls = Cell::new(0_usize);
    assert!(store
        .remove_while(&record.key, || count(&calls, None))
        .unwrap());
    assert!(calls.get() > 10);
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

fn cache_directory(root: &std::path::Path, key: &ContentDigest) -> std::path::PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}
