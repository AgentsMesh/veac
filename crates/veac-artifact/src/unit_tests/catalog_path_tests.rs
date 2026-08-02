use std::fs;

use crate::{ArtifactCatalogQuery, ArtifactErrorKind, ArtifactKind, ArtifactStore};

#[test]
fn catalog_rejects_malformed_prefixes_suffixes_and_non_directories() {
    for layout in [Layout::BadPrefix, Layout::BadSuffix, Layout::FileDigestRoot] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("store");
        fs::create_dir(&root).unwrap();
        match layout {
            Layout::BadPrefix => fs::create_dir_all(root.join("sha256/gg")).unwrap(),
            Layout::BadSuffix => {
                fs::create_dir_all(root.join(format!("sha256/aa/{}", "b".repeat(61)))).unwrap()
            }
            Layout::FileDigestRoot => fs::write(root.join("sha256"), b"file").unwrap(),
        }
        assert_eq!(
            ArtifactStore::new(root).catalog(&query()).unwrap_err().kind,
            ArtifactErrorKind::CorruptCache
        );
    }
}

#[cfg(unix)]
#[test]
fn catalog_rejects_non_utf8_entry_names() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = tempfile::tempdir().unwrap();
    let digest_root = temp.path().join("store/sha256");
    fs::create_dir_all(&digest_root).unwrap();
    if fs::create_dir(digest_root.join(OsString::from_vec(vec![0xff]))).is_err() {
        // APFS rejects non-UTF-8 names before the catalog can observe them.
        return;
    }
    assert_eq!(
        ArtifactStore::new(temp.path().join("store"))
            .catalog(&query())
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
}

fn query() -> ArtifactCatalogQuery {
    ArtifactCatalogQuery::new(
        ArtifactKind::ProxyVideo,
        vec![crate::ArtifactDependency {
            role: "source".into(),
            identity: crate::ContentDigest::sha256(b"source"),
        }],
    )
    .unwrap()
}

enum Layout {
    BadPrefix,
    BadSuffix,
    FileDigestRoot,
}
