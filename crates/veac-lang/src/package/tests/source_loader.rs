use std::path::{Path, PathBuf};

use super::*;

fn snapshot(root: &Path, entry: PathBuf, source: &[u8]) -> PackageDiscovery {
    let package = identity("root", "1.0.0");
    let file = LockedFile {
        path: "main.veac".to_owned(),
        sha256: sha256_bytes(source),
    };
    let lock = PackageLockV1::new(package.clone(), vec![file], vec![]).unwrap();
    PackageDiscovery {
        root: DiscoveredPackage {
            package: package.clone(),
            path: root.to_path_buf(),
            entry,
            api: api::ApiMetadataV1::new(package, vec![]),
            entry_sha256: sha256_bytes(source),
            content_sha256: lock.root_content_sha256.clone(),
            dependencies: vec![],
        },
        dependencies: vec![],
        lock,
    }
}

fn loader(discovery: &PackageDiscovery) -> PackageSourceLoader {
    source_loader::PackageSourceLoader::from_discovery(discovery).unwrap()
}

#[test]
fn loader_snapshot_rejects_file_identity_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("main.veac");
    std::fs::write(&path, b"module {}\n").unwrap();
    let discovery = snapshot(temp.path(), path.clone(), b"module {}\n");
    let loader = loader(&discovery);
    let old = temp.path().join("old.veac");
    std::fs::rename(&path, old).unwrap();
    std::fs::write(&path, b"module {}\n").unwrap();

    let error = loader.load_entry().unwrap_err();
    assert_eq!(error.kind(), PackageErrorKind::Contract);
    assert!(error.message().contains("source identity changed"));
}

#[test]
fn loader_snapshot_reports_source_utf8_failure_after_digest_verification() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("main.veac");
    std::fs::write(&path, [0xff]).unwrap();
    let discovery = snapshot(temp.path(), path, &[0xff]);

    let error = loader(&discovery).load_entry().unwrap_err();
    assert_eq!(error.kind(), PackageErrorKind::Io);
    assert!(error
        .message()
        .contains("source packages/root@1.0.0/main.veac is not UTF-8"));
}

#[test]
fn loader_snapshot_rejects_escaped_missing_and_unlocked_entries() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("main.veac");
    std::fs::write(&path, b"module {}\n").unwrap();

    let escaped = snapshot(temp.path(), PathBuf::from("/outside.veac"), b"module {}\n");
    let error = source_loader::PackageSourceLoader::from_discovery(&escaped).unwrap_err();
    assert!(error.message().contains("entry escapes its root"));

    let unlocked = snapshot(temp.path(), temp.path().join("other.veac"), b"module {}\n");
    let error = source_loader::PackageSourceLoader::from_discovery(&unlocked).unwrap_err();
    assert!(error.message().contains("source is not declared"));

    let missing = snapshot(
        &temp.path().join("missing"),
        temp.path().join("missing/main.veac"),
        b"module {}\n",
    );
    assert_eq!(
        source_loader::PackageSourceLoader::from_discovery(&missing)
            .unwrap_err()
            .kind(),
        PackageErrorKind::Io
    );
}

#[test]
fn qualified_loader_enforces_importer_edges_and_exposes_identity() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("main.veac");
    std::fs::write(&path, b"module {}\n").unwrap();
    let discovery = snapshot(temp.path(), path, b"module {}\n");
    let loader = loader(&discovery);
    let root = identity("root", "1.0.0");

    assert_eq!(loader.identity(), &root);
    assert_eq!(
        loader
            .load_qualified("packages/root@1.0.0/main.veac", &root, "./main.veac")
            .unwrap()
            .id,
        "packages/root@1.0.0/main.veac"
    );
    let missing = identity("missing", "1.0.0");
    assert!(loader
        .load_qualified("packages/root@1.0.0/main.veac", &missing, "main.veac")
        .is_err());
    assert!(loader
        .load_qualified("unknown.veac", &root, "main.veac")
        .is_err());
}

#[cfg(unix)]
#[test]
fn relative_entry_must_be_utf8() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let root = Path::new("/tmp/package");
    let path = root.join(OsString::from_vec(vec![0xff]));
    let error = source_loader::relative_id(root, &path).unwrap_err();
    assert!(error.message().contains("entry is not valid UTF-8"));
}
