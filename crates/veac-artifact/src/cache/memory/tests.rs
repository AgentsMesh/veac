use super::*;
use crate::test_support;

#[test]
fn conflict_reverification_rejects_a_vanished_winner() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let descriptor = test_support::descriptor();
    let expected = ArtifactRecord {
        key: artifact_key(&descriptor).unwrap(),
        content: ContentDigest::sha256(b"payload"),
        size_bytes: 7,
    };

    let error = store
        .reuse_winner(&descriptor, &expected, || true)
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::Io);
}
