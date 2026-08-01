use std::path::{Path, PathBuf};

use super::*;
use veac_artifact::{ContentDigest, DeliveryPackageMember};

#[cfg(unix)]
#[test]
fn rejects_a_non_utf8_entrypoint() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("file"), b"data").unwrap();
    let root = Directory::open(temp.path()).unwrap();
    let entrypoint = PathBuf::from(OsString::from_vec(vec![0xff]));
    let error = inventory(&root, &entrypoint, limit()).unwrap_err();
    assert!(error.message.contains("entrypoint must be valid UTF-8"));
}

#[test]
fn rejects_overlong_member_paths_and_empty_regular_files() {
    let temp = tempfile::tempdir().unwrap();
    let directory = "a".repeat(120);
    let child = "b".repeat(120);
    std::fs::create_dir(temp.path().join(&directory)).unwrap();
    std::fs::write(temp.path().join(directory).join(child), b"data").unwrap();
    let root = Directory::open(temp.path()).unwrap();
    let error = inventory(&root, Path::new("missing"), limit()).unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::ResourceLimit);
    assert!(error.message.contains("member shape exceeds its budget"));

    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("empty"), b"").unwrap();
    let root = Directory::open(temp.path()).unwrap();
    let error = inventory(&root, Path::new("empty"), limit()).unwrap_err();
    assert!(error.message.contains("regular files must be non-empty"));
}

#[test]
fn preserves_artifact_error_classification() {
    let limited = veac_artifact::DeliveryPackageInventory::new("entry", Vec::new()).unwrap_err();
    assert_eq!(
        artifact_error(limited).kind,
        crate::RuntimeErrorKind::ResourceLimit
    );

    let member = DeliveryPackageMember {
        path: "other".into(),
        node_type: DeliveryPackageNodeType::RegularFile,
        size_bytes: 1,
        content: Some(ContentDigest::sha256(b"x")),
    };
    let invalid = veac_artifact::DeliveryPackageInventory::new("entry", vec![member]).unwrap_err();
    let error = artifact_error(invalid);
    assert_eq!(error.kind, crate::RuntimeErrorKind::General);
    assert!(error.message.contains("invalid delivery package"));
}

fn limit() -> Instant {
    Instant::now() + std::time::Duration::from_secs(10)
}
