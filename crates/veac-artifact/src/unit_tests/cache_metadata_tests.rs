use std::fs;

use crate::{test_support::descriptor, *};

#[test]
fn cache_rejects_semantically_invalid_descriptor_metadata() {
    let (temp, store, record) = stored();
    let directory = cache_directory(temp.path(), &record.key);
    let mut invalid = descriptor();
    invalid.producer.name.clear();
    fs::write(
        directory.join("descriptor.json"),
        serde_json_canonicalizer::to_vec(&invalid).unwrap(),
    )
    .unwrap();
    assert_corrupt(store.get(&record.key));
}

#[test]
fn verified_open_rejects_a_descriptor_from_another_key() {
    let (temp, store, record) = stored();
    let directory = cache_directory(temp.path(), &record.key);
    let mut other = descriptor();
    other.parameters = serde_json::json!({"codec": "h265", "height": 720});
    fs::write(
        directory.join("descriptor.json"),
        canonical_descriptor_bytes(&other).unwrap(),
    )
    .unwrap();
    assert_corrupt(store.open(&record.key));
}

#[test]
fn cache_get_rejects_a_descriptor_or_record_from_another_key() {
    let (temp, store, record) = stored();
    let directory = cache_directory(temp.path(), &record.key);
    let mut other = descriptor();
    other.parameters = serde_json::json!({"codec": "h265", "height": 720});
    fs::write(
        directory.join("descriptor.json"),
        canonical_descriptor_bytes(&other).unwrap(),
    )
    .unwrap();
    assert_corrupt(store.get(&record.key));

    let (temp, store, record) = stored();
    let directory = cache_directory(temp.path(), &record.key);
    let other = ArtifactRecord {
        key: ContentDigest::sha256(b"another key"),
        ..record.clone()
    };
    fs::write(
        directory.join("record.json"),
        serde_json_canonicalizer::to_vec(&other).unwrap(),
    )
    .unwrap();
    assert_corrupt(store.get(&record.key));
}

#[test]
fn cache_rejects_invalid_record_digests_and_oversized_lengths() {
    for mutate in 0..3 {
        let (temp, store, record) = stored();
        let directory = cache_directory(temp.path(), &record.key);
        let mut invalid = record.clone();
        match mutate {
            0 => invalid.key.value = "bad".into(),
            1 => invalid.content.value = "bad".into(),
            _ => invalid.size_bytes = veac_ir::MAX_SAFE_INTEGER + 1,
        }
        fs::write(
            directory.join("record.json"),
            serde_json_canonicalizer::to_vec(&invalid).unwrap(),
        )
        .unwrap();
        assert_corrupt(store.get(&record.key));
    }
}

#[test]
fn cache_rejects_a_non_file_payload_and_content_metadata_drift() {
    let (temp, store, record) = stored();
    let directory = cache_directory(temp.path(), &record.key);
    fs::remove_file(directory.join("payload.bin")).unwrap();
    fs::create_dir(directory.join("payload.bin")).unwrap();
    assert_corrupt(store.get(&record.key));

    let (temp, store, record) = stored();
    let directory = cache_directory(temp.path(), &record.key);
    let invalid = ArtifactRecord {
        content: ContentDigest::sha256(b"other"),
        ..record.clone()
    };
    fs::write(
        directory.join("record.json"),
        serde_json_canonicalizer::to_vec(&invalid).unwrap(),
    )
    .unwrap();
    assert_corrupt(store.get(&record.key));
}

fn stored() -> (tempfile::TempDir, ArtifactStore, ArtifactRecord) {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store.put(&descriptor(), b"payload").unwrap();
    (temp, store, record)
}

fn assert_corrupt<T: std::fmt::Debug>(result: ArtifactResult<T>) {
    assert_eq!(result.unwrap_err().kind, ArtifactErrorKind::CorruptCache);
}

fn cache_directory(root: &std::path::Path, key: &ContentDigest) -> std::path::PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}
