use std::os::unix::ffi::OsStrExt;

use super::*;

#[test]
fn catalog_text_and_orphan_checks_reject_unrepresentable_or_missing_entries() {
    assert_eq!(
        text(OsStr::from_bytes(b"\xff")).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    std::fs::create_dir_all(root.join("sha256/aa")).unwrap();
    let mut prefix = BoundChain::root(&root, false).unwrap().unwrap();
    prefix
        .extend(&["sha256".into(), "aa".into()], false)
        .unwrap();
    assert_eq!(
        require_orphan(
            &prefix,
            OsStr::new(".veac-cache-00000000000000000000000000000000"),
        )
        .unwrap_err()
        .kind,
        ArtifactErrorKind::CorruptCache
    );
}
