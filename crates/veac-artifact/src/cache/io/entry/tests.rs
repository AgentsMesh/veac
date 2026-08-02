use super::*;
use crate::ArtifactErrorKind;

#[test]
fn bound_file_open_create_and_reopen_errors_are_classified() {
    let temp = tempfile::tempdir().unwrap();
    let directory = File::open(temp.path()).unwrap();
    assert_eq!(
        BoundFile::open(&directory, OsStr::new("missing"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );

    let path = temp.path().join("entry");
    std::fs::write(&path, b"payload").unwrap();
    assert_eq!(
        BoundFile::create(&directory, OsStr::new("entry"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::Io
    );
    let bound = BoundFile::open(&directory, OsStr::new("entry")).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(
        bound.verify_while(&directory, || true).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
}

#[test]
fn bound_file_read_and_fingerprint_limits_fail_before_returning_bytes() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("entry"), b"payload").unwrap();
    let directory = File::open(temp.path()).unwrap();
    let mut bound = BoundFile::open(&directory, OsStr::new("entry")).unwrap();
    assert_eq!(
        bound
            .read_bounded_while(3, "too large", || true)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::ResourceLimit
    );
    assert_eq!(
        bound.fingerprint_while(3, || true).unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );
}

#[test]
fn bound_files_reject_hard_linked_entries() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("entry");
    std::fs::write(&path, b"payload").unwrap();
    std::fs::hard_link(&path, temp.path().join("alias")).unwrap();
    let directory = File::open(temp.path()).unwrap();
    assert_eq!(
        BoundFile::open(&directory, OsStr::new("entry"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
}

#[test]
fn unlink_reports_a_directory_permission_failure() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("entry");
    std::fs::write(&path, b"payload").unwrap();
    let directory = File::open(temp.path()).unwrap();
    let bound = BoundFile::open(&directory, OsStr::new("entry")).unwrap();
    std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
    let error = bound.unlink(&directory).unwrap_err();
    std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();

    assert_eq!(error.kind, ArtifactErrorKind::Io);
    std::fs::remove_file(path).unwrap();
}
