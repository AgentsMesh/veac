use std::cell::Cell;
use std::fs;
use std::path::Path;

use crate::{test_support, *};

#[test]
fn guarded_file_store_checks_chunks_and_cleans_mid_write_cancellation() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("payload.bin");
    let payload = vec![0x3c; 2 * 1024 * 1024 + 31];
    fs::write(&source, &payload).unwrap();
    let descriptor = test_support::descriptor();
    let content = ContentDigest::sha256(&payload);

    let success = ArtifactStore::new(temp.path().join("success"));
    let calls = Cell::new(0_usize);
    let record = success
        .put_file_expected_while(&descriptor, &source, &content, payload.len() as u64, || {
            count(&calls, None)
        })
        .unwrap();
    assert_eq!(record.content, content);
    assert!(calls.get() > 256);

    let cancelled = ArtifactStore::new(temp.path().join("cancelled"));
    let current = Cell::new(0_usize);
    let error = cancelled
        .put_file_expected_while(&descriptor, &source, &content, payload.len() as u64, || {
            count(&current, Some(calls.get() / 2))
        })
        .unwrap_err();
    assert_resource(error);
    assert!(cancelled
        .get(&artifact_key(&descriptor).unwrap())
        .unwrap()
        .is_none());
    assert!(!cache_directory(cancelled.root(), &artifact_key(&descriptor).unwrap()).exists());
}

#[test]
fn verified_file_store_validates_declarations_before_source_access() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let key = artifact_key(&descriptor).unwrap();
    let missing = temp.path().join("missing");
    let declared = ArtifactRecord {
        key: key.clone(),
        content: ContentDigest::sha256(b"payload"),
        size_bytes: 7,
    };

    assert_resource(
        store
            .put_verified_file_while(&descriptor, &declared, &missing, || false)
            .unwrap_err(),
    );
    let wrong_key = ArtifactRecord {
        key: ContentDigest::sha256(b"wrong"),
        ..declared.clone()
    };
    assert_eq!(
        store
            .put_verified_file_while(&descriptor, &wrong_key, &missing, || true)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::IdentityMismatch
    );
    let invalid_content = ArtifactRecord {
        content: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: "bad".into(),
        },
        ..declared.clone()
    };
    assert_eq!(
        store
            .put_verified_file_while(&descriptor, &invalid_content, &missing, || true)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );
    let oversized = ArtifactRecord {
        size_bytes: MAX_ARTIFACT_PAYLOAD_BYTES + 1,
        ..declared
    };
    assert_eq!(
        store
            .put_verified_file_while(&descriptor, &oversized, &missing, || true)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::ResourceLimit
    );
}

#[test]
fn verified_file_store_reports_fresh_and_reused_commit_states_exactly() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("payload.bin");
    fs::write(&source, b"payload").unwrap();
    let descriptor = test_support::descriptor();
    let declared = ArtifactRecord {
        key: artifact_key(&descriptor).unwrap(),
        content: ContentDigest::sha256(b"payload"),
        size_bytes: 7,
    };
    let fresh = ArtifactStore::new(temp.path().join("fresh"));
    let target = cache_directory(fresh.root(), &declared.key);
    let error = fresh
        .put_verified_file_while(&descriptor, &declared, &source, || {
            !target.join("sealed").exists()
        })
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert_eq!(error.commit_state, ArtifactCommitState::Committed);
    assert!(fresh.open(&declared.key).unwrap().is_some());

    let reused = ArtifactStore::new(temp.path().join("reused"));
    reused.put_file(&descriptor, &source).unwrap();
    let calls = Cell::new(0_usize);
    reused
        .put_verified_file_while(&descriptor, &declared, &source, || count(&calls, None))
        .unwrap();
    let current = Cell::new(0_usize);
    let error = reused
        .put_verified_file_while(&descriptor, &declared, &source, || {
            count(&current, Some(calls.get()))
        })
        .unwrap_err();
    assert_resource(error);
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

fn cache_directory(root: &Path, key: &ContentDigest) -> std::path::PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}
