use std::fs;

use crate::{test_support::descriptor, *};

#[test]
fn streamed_file_storage_verifies_reuses_and_rejects_content_drift() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("master.mp4");
    fs::write(&source, vec![0x5a; 256 * 1024]).unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let expected = descriptor();
    let first = store.put_file(&expected, &source).unwrap();
    assert_eq!(first.size_bytes, 256 * 1024);
    assert_eq!(store.put_file(&expected, &source).unwrap(), first);
    let opened = store.open_verified(&first.key, &expected).unwrap().unwrap();
    assert_eq!(opened.record(), &first);

    fs::write(&source, b"different deterministic output").unwrap();
    assert_eq!(
        store.put_file(&expected, &source).unwrap_err().kind,
        ArtifactErrorKind::IdentityMismatch
    );
}

#[test]
fn expected_file_storage_rejects_post_render_replacement_before_promotion() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("master.mp4");
    let rendered = b"runtime-verified-render";
    fs::write(&source, rendered).unwrap();
    let expected = ContentDigest::sha256(rendered);
    fs::write(&source, b"replaced-untrusted-file").unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = descriptor();

    assert_eq!(
        store
            .put_file_expected(&descriptor, &source, &expected, rendered.len() as u64)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::IdentityMismatch
    );
    assert!(store
        .get(&artifact_key(&descriptor).unwrap())
        .unwrap()
        .is_none());
}

#[cfg(unix)]
#[test]
fn streamed_file_storage_rejects_symlink_sources() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let linked = temp.path().join("linked");
    fs::write(&source, b"payload").unwrap();
    symlink(&source, &linked).unwrap();
    assert_eq!(
        ArtifactStore::new(temp.path().join("store"))
            .put_file(&descriptor(), &linked)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
}

#[test]
fn streamed_file_storage_rejects_missing_and_directory_sources() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    assert_eq!(
        store
            .put_file(&descriptor(), &temp.path().join("missing"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::Io
    );
    assert_eq!(
        store.put_file(&descriptor(), temp.path()).unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
}

#[test]
fn large_payloads_require_streaming_and_never_enter_whole_vec_cache_paths() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("payload.bin");
    let payload = vec![0x5a; MAX_IN_MEMORY_ARTIFACT_BYTES as usize + 1];
    let descriptor = descriptor();
    let store = ArtifactStore::new(temp.path().join("store"));
    assert_eq!(
        store.put(&descriptor, &payload).unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );

    drop(payload);
    fs::write(&source, b"streamed payload").unwrap();
    let record = store.put_file(&descriptor, &source).unwrap();
    assert_eq!(
        store
            .put_verified_file(&descriptor, &record, &source)
            .unwrap(),
        record
    );

    let directory = store
        .root()
        .join("sha256")
        .join(&record.key.value[..2])
        .join(&record.key.value[2..]);
    fs::OpenOptions::new()
        .write(true)
        .open(directory.join("payload.bin"))
        .unwrap()
        .set_len(MAX_IN_MEMORY_ARTIFACT_BYTES + 1)
        .unwrap();
    let mut oversized = record;
    oversized.size_bytes = MAX_IN_MEMORY_ARTIFACT_BYTES + 1;
    fs::write(
        directory.join("record.json"),
        serde_json_canonicalizer::to_vec(&oversized).unwrap(),
    )
    .unwrap();
    assert_eq!(
        store.get(&oversized.key).unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );
}

#[test]
fn declared_streaming_payload_cannot_raise_the_disk_cap() {
    let temp = tempfile::tempdir().unwrap();
    let error = ArtifactStore::new(temp.path().join("store"))
        .put_file_expected(
            &descriptor(),
            &temp.path().join("missing"),
            &ContentDigest::sha256(b"payload"),
            MAX_ARTIFACT_PAYLOAD_BYTES + 1,
        )
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
}
