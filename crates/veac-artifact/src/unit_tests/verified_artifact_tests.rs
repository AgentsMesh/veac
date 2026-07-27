use std::fs;

use serde_json::json;

use crate::{test_support, *};

#[test]
fn exact_open_returns_open_time_verified_metadata_and_path() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, b"proxy payload").unwrap();
    let opened = store
        .open_verified(&record.key, &descriptor)
        .unwrap()
        .unwrap();
    assert_eq!(opened.descriptor(), &descriptor);
    assert_eq!(opened.record(), &record);
    assert_eq!(
        opened.payload_path(),
        fs::canonicalize(cache_directory(store.root(), &record.key).join("payload.bin")).unwrap()
    );
    assert_eq!(fs::read(opened.payload_path()).unwrap(), b"proxy payload");
    let opened_without_expectation = store.open(&record.key).unwrap().unwrap();
    assert_eq!(opened_without_expectation, opened);
}

#[test]
fn exact_open_distinguishes_a_miss_from_an_invalid_expectation() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let expected = test_support::descriptor();
    let expected_key = artifact_key(&expected).unwrap();
    assert!(store
        .open_verified(&expected_key, &expected)
        .unwrap()
        .is_none());
    let invalid = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: "bad".into(),
    };
    assert_eq!(
        store.open_verified(&invalid, &expected).unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );

    let mut other = expected.clone();
    other.parameters = json!({"codec": "h265", "height": 720});
    assert_eq!(
        store.open_verified(&expected_key, &other).unwrap_err().kind,
        ArtifactErrorKind::IdentityMismatch
    );
}

#[test]
fn bounded_exact_open_rejects_before_hashing_and_cannot_raise_the_hard_cap() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, b"payload").unwrap();
    assert!(store
        .open_verified_bounded(&record.key, &descriptor, record.size_bytes)
        .unwrap()
        .is_some());
    fs::write(
        cache_directory(store.root(), &record.key).join("payload.bin"),
        b"corrupt",
    )
    .unwrap();
    assert_eq!(
        store
            .open_verified_bounded(&record.key, &descriptor, record.size_bytes - 1)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::ResourceLimit
    );
    for limit in [0, MAX_ARTIFACT_PAYLOAD_BYTES + 1, u64::MAX] {
        assert_eq!(
            store
                .open_verified_bounded(&record.key, &descriptor, limit)
                .unwrap_err()
                .kind,
            ArtifactErrorKind::InvalidContract
        );
    }
}

#[test]
fn exact_open_rejects_noncanonical_descriptor_and_record_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, b"payload").unwrap();
    let directory = cache_directory(store.root(), &record.key);

    let canonical = canonical_descriptor_bytes(&descriptor).unwrap();
    let mut padded = Vec::with_capacity(canonical.len() + 1);
    padded.push(b' ');
    padded.extend(canonical);
    fs::write(directory.join("descriptor.json"), padded).unwrap();
    assert_eq!(
        store
            .open_verified(&record.key, &descriptor)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );

    fs::write(
        directory.join("descriptor.json"),
        canonical_descriptor_bytes(&descriptor).unwrap(),
    )
    .unwrap();
    let mut record_bytes = serde_json_canonicalizer::to_vec(&record).unwrap();
    record_bytes.push(b'\n');
    fs::write(directory.join("record.json"), record_bytes).unwrap();
    assert_eq!(
        store
            .open_verified(&record.key, &descriptor)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
}

#[test]
fn exact_open_rejects_payload_digest_and_size_changes() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, b"payload").unwrap();
    let payload = cache_directory(store.root(), &record.key).join("payload.bin");
    fs::write(&payload, b"changed").unwrap();
    assert_eq!(
        store
            .open_verified(&record.key, &descriptor)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
    fs::write(&payload, b"payload plus bytes").unwrap();
    assert_eq!(
        store
            .open_verified(&record.key, &descriptor)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
}

#[cfg(unix)]
#[test]
fn exact_open_rejects_a_symlink_payload() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let record = store.put(&descriptor, b"payload").unwrap();
    let payload = cache_directory(store.root(), &record.key).join("payload.bin");
    let external = temp.path().join("external");
    fs::write(&external, b"payload").unwrap();
    fs::remove_file(&payload).unwrap();
    symlink(external, payload).unwrap();
    assert_eq!(
        store
            .open_verified(&record.key, &descriptor)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
}

fn cache_directory(root: &std::path::Path, key: &ContentDigest) -> std::path::PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}
