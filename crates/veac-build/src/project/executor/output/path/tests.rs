use super::{checked_directory, checked_file, fingerprint};
use crate::{CancellationToken, ExecutionErrorKind};

#[test]
fn output_paths_accept_expected_kinds_and_reject_invalid_shapes() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    std::fs::create_dir(&workspace).unwrap();
    std::fs::write(workspace.join("file"), b"payload").unwrap();
    std::fs::create_dir(workspace.join("directory")).unwrap();

    assert!(checked_file(&workspace, std::path::Path::new("file")).is_ok());
    assert!(checked_directory(&workspace, std::path::Path::new("directory")).is_ok());
    assert!(checked_file(&workspace, std::path::Path::new("directory")).is_err());
    assert!(checked_directory(&workspace, std::path::Path::new("file")).is_err());
    for value in ["", "../escape", "/absolute"] {
        assert!(checked_file(&workspace, std::path::Path::new(value)).is_err());
    }
    assert!(checked_file(&workspace, std::path::Path::new("missing")).is_err());
    assert!(checked_file(
        &temp.path().join("missing-workspace"),
        std::path::Path::new("file")
    )
    .is_err());
}

#[test]
fn output_fingerprinting_is_content_based_and_cancellable() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("file");
    std::fs::write(&file, b"payload").unwrap();
    let (digest, size) = fingerprint(&file, &CancellationToken::new()).unwrap();
    assert_eq!(size, 7);
    assert_eq!(digest, crate::ContentDigest::sha256(b"payload"));

    let token = CancellationToken::new();
    token.cancel();
    let error = fingerprint(&file, &token).unwrap_err();
    assert_eq!(error.kind(), ExecutionErrorKind::Cancelled);
    assert!(fingerprint(&temp.path().join("missing"), &CancellationToken::new()).is_err());
}

#[cfg(unix)]
#[test]
fn output_paths_reject_symlink_components() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&workspace).unwrap();
    std::fs::write(&outside, b"x").unwrap();
    std::os::unix::fs::symlink(&outside, workspace.join("link")).unwrap();
    assert!(checked_file(&workspace, std::path::Path::new("link")).is_err());
}
