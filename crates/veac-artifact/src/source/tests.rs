use super::*;
use crate::ArtifactErrorKind;

#[test]
fn relative_copy_destination_uses_a_guarded_current_directory_stage() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::write(&source, b"x").unwrap();
    let destination = Path::new("guarded-relative-output");

    let error = copy_with_limit_while(&source, destination, None, 1, || {}, || false).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert!(!destination.exists());
}
