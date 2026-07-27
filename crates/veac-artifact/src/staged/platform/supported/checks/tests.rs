use super::*;

#[test]
fn missing_directories_and_unsupported_publication_are_classified() {
    let temp = tempfile::tempdir().unwrap();
    assert_eq!(
        open_directory(&temp.path().join("missing"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::Io
    );
    assert_eq!(
        publish_error(PublishMode::NoClobber, rustix::io::Errno::NOSYS).kind,
        ArtifactErrorKind::UnsafePath
    );
    assert_eq!(
        publish_error(PublishMode::Replace, rustix::io::Errno::NOSYS).kind,
        ArtifactErrorKind::Io
    );
}

#[test]
fn identity_checks_reject_a_different_open_file() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    std::fs::write(&first, b"first").unwrap();
    std::fs::write(&second, b"second").unwrap();
    let first = File::open(first).unwrap();
    let second = File::open(second).unwrap();
    assert_eq!(
        verify_identity(&first, &second, FileType::RegularFile)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
}
