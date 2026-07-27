use std::fs;
use std::os::unix::fs::symlink;

use crate::{artifact_key, test_support, ArtifactErrorKind, ArtifactRecord, ContentDigest};

use super::super::{inspect, operations};

#[test]
fn read_rejects_a_payload_replaced_after_its_descriptor_is_opened() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let descriptor = test_support::descriptor();
    let key = artifact_key(&descriptor).unwrap();
    let directory = root
        .join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..]);
    let record = ArtifactRecord {
        key,
        content: ContentDigest::sha256(b"payload"),
        size_bytes: 7,
    };
    operations::write_atomic_while_with(
        &root,
        &directory,
        &descriptor,
        &record,
        b"payload",
        |_| {},
        |_| {},
        |_| {},
        || true,
    )
    .unwrap();
    let external = temp.path().join("external");
    fs::write(&external, b"replacement must not be read").unwrap();
    let moved = temp.path().join("opened-payload");
    let error = inspect::inspect_bounded_while_with(
        &root,
        &directory,
        crate::MAX_ARTIFACT_METADATA_BYTES,
        |artifact| {
            fs::rename(artifact.join("payload.bin"), &moved).unwrap();
            symlink(&external, artifact.join("payload.bin")).unwrap();
        },
        || true,
    )
    .err()
    .expect("entry replacement must fail closed");
    assert_eq!(error.kind, ArtifactErrorKind::CorruptCache);
    assert_eq!(fs::read(moved).unwrap(), b"payload");
    assert_eq!(fs::read(external).unwrap(), b"replacement must not be read");
}
