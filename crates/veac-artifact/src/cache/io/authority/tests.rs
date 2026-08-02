use super::*;
use crate::ArtifactErrorKind;

#[test]
fn cache_directory_creation_reuses_directories_and_rejects_files() {
    let temp = tempfile::tempdir().unwrap();
    let parent = File::open(temp.path()).unwrap();
    std::fs::create_dir(temp.path().join("existing")).unwrap();
    create_directory(&parent, OsStr::new("existing")).unwrap();

    std::fs::write(temp.path().join("regular"), b"regular").unwrap();
    assert_eq!(
        create_directory(&parent, OsStr::new("regular"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
    let regular = File::open(temp.path().join("regular")).unwrap();
    assert_eq!(
        create_directory(&regular, OsStr::new("child"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::Io
    );
}

#[test]
fn cache_identity_checks_reject_different_open_files() {
    let first = tempfile::NamedTempFile::new().unwrap();
    let second = tempfile::NamedTempFile::new().unwrap();
    assert_eq!(
        verify_identity(first.as_file(), second.as_file(), FileType::RegularFile)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
}
