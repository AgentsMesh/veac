use std::fs;
use std::os::unix::fs::symlink;

use crate::{artifact_key, test_support, ArtifactErrorKind, ArtifactRecord, ContentDigest};

use super::super::operations;

#[test]
fn write_rejects_a_replaced_symlinked_root_ancestor() {
    let temp = tempfile::tempdir().unwrap();
    let real = temp.path().join("real");
    let external = temp.path().join("external");
    fs::create_dir(&real).unwrap();
    fs::create_dir(&external).unwrap();
    fs::write(external.join("victim"), b"safe").unwrap();
    let linked = temp.path().join("linked");
    symlink("real", &linked).unwrap();
    let root = linked.join("store");
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
    let error = operations::write_atomic_while_with(
        &root,
        &directory,
        &descriptor,
        &record,
        b"payload",
        |_| {
            fs::remove_file(&linked).unwrap();
            symlink("external", &linked).unwrap();
        },
        |_| {},
        |_| {},
        || true,
    )
    .unwrap_err();
    assert!(matches!(
        error.kind,
        ArtifactErrorKind::CorruptCache | ArtifactErrorKind::UnsafePath
    ));
    assert_eq!(fs::read(external.join("victim")).unwrap(), b"safe");
    assert!(!external.join("store").exists());
}
