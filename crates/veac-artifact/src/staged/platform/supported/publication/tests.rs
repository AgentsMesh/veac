use std::os::unix::fs::PermissionsExt;

use super::*;

#[test]
fn destination_and_published_entry_reverification_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let stage = Stage::new(temp.path()).unwrap();
    assert_eq!(
        stage.destination_name(Path::new("/")).unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
    assert_eq!(
        stage
            .verify_published(OsStr::new("missing"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
}

#[test]
fn cleanup_directory_refuses_to_remove_a_nonempty_stage() {
    let temp = tempfile::tempdir().unwrap();
    let mut stage = Stage::new(temp.path()).unwrap();
    assert_eq!(
        stage.cleanup_directory().unwrap_err().kind,
        ArtifactErrorKind::Io
    );
}

#[test]
fn payload_reverification_rejects_a_removed_visible_entry() {
    let temp = tempfile::tempdir().unwrap();
    let stage = Stage::new(temp.path()).unwrap();
    std::fs::remove_file(stage.path()).unwrap();
    assert_eq!(
        stage.verify_payload_identity().unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
}

#[test]
fn discard_surfaces_unlink_permission_failures_and_can_retry() {
    let temp = tempfile::tempdir().unwrap();
    let mut stage = Stage::new(temp.path()).unwrap();
    let directory = stage.path().parent().unwrap().to_path_buf();
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o500)).unwrap();
    let error = stage.discard().unwrap_err();
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();

    assert_eq!(error.kind, ArtifactErrorKind::Io);
    stage.discard().unwrap();
}
