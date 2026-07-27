use std::fs;

use crate::{test_support::descriptor, *};

#[test]
fn store_round_trips_and_reuses_identical_artifacts() {
    let root = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(root.path());
    let record = store.put(&descriptor(), b"proxy bytes").unwrap();
    let reused = store.put(&descriptor(), b"proxy bytes").unwrap();
    assert_eq!(record, reused);
    let cached = store.get(&record.key).unwrap().unwrap();
    assert_eq!(cached.record, record);
    assert_eq!(cached.descriptor, descriptor());
    assert_eq!(cached.payload, b"proxy bytes");
    assert!(store.remove(&record.key).unwrap());
    assert!(!store.remove(&record.key).unwrap());
    assert!(store.get(&record.key).unwrap().is_none());
}

#[test]
fn store_rejects_nondeterministic_output_for_the_same_key() {
    let root = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(root.path());
    store.put(&descriptor(), b"first").unwrap();
    assert_eq!(
        store.put(&descriptor(), b"second").unwrap_err().kind,
        ArtifactErrorKind::IdentityMismatch
    );
}

#[test]
fn invalid_keys_and_unwritable_roots_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(root.path());
    let invalid = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: "bad".to_owned(),
    };
    assert_eq!(
        store.get(&invalid).unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
    assert_eq!(
        store.remove(&invalid).unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
    let file_root = root.path().join("file");
    fs::write(&file_root, b"not a directory").unwrap();
    assert_eq!(
        ArtifactStore::new(file_root)
            .put(&descriptor(), b"data")
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
}

#[test]
fn cache_corruption_is_detected_for_payload_and_metadata() {
    let root = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(root.path());
    let record = store.put(&descriptor(), b"payload").unwrap();
    let directory = cache_directory(root.path(), &record.key);
    fs::write(directory.join("payload.bin"), b"corrupt").unwrap();
    assert_eq!(
        store.get(&record.key).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
    fs::write(directory.join("descriptor.json"), b"not-json").unwrap();
    let error = store.get(&record.key).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::CorruptCache);
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn cache_rejects_a_record_with_the_wrong_key() {
    let root = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(root.path());
    let record = store.put(&descriptor(), b"payload").unwrap();
    let directory = cache_directory(root.path(), &record.key);
    let wrong = ArtifactRecord {
        key: ContentDigest::sha256(b"wrong"),
        ..record.clone()
    };
    fs::write(
        directory.join("record.json"),
        serde_json::to_vec(&wrong).unwrap(),
    )
    .unwrap();
    assert_eq!(
        store.get(&record.key).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );

    let wrong_size = ArtifactRecord {
        size_bytes: record.size_bytes + 1,
        ..record.clone()
    };
    fs::write(
        directory.join("record.json"),
        serde_json::to_vec(&wrong_size).unwrap(),
    )
    .unwrap();
    assert_eq!(
        store.get(&record.key).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );

    fs::write(directory.join("record.json"), b"not-json").unwrap();
    let error = store.get(&record.key).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::CorruptCache);
    assert!(std::error::Error::source(&error).is_some());
}

fn cache_directory(root: &std::path::Path, key: &ContentDigest) -> std::path::PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}
