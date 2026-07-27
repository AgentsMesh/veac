use std::io::Write;

use super::*;
use crate::{test_support, ArtifactErrorKind};

#[test]
fn payload_and_private_directory_reverification_detect_disappearance() {
    let temp = tempfile::tempdir().unwrap();
    let stage = Stage::new(temp.path()).unwrap();
    std::fs::remove_file(stage.path()).unwrap();
    assert_eq!(
        stage.verify_payload().unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );

    let stage = Stage::new(temp.path()).unwrap();
    let directory = stage.path().parent().unwrap().to_path_buf();
    std::fs::rename(&directory, temp.path().join("moved")).unwrap();
    assert_eq!(
        stage.verify_directory().unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
}

#[test]
fn relative_destination_parent_and_content_paths_are_guarded() {
    let temp = tempfile::tempdir().unwrap();
    let mut stage = Stage::new(temp.path()).unwrap();
    stage.file_mut().write_all(b"content").unwrap();
    assert_eq!(
        stage
            .verify_destination_parent(Path::new("output"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
    let content = stage.content_while(|| true).unwrap();
    assert_eq!(
        content.sha256,
        test_support::media_identity(b"content").digest
    );
    assert_eq!(content.size_bytes, 7);
    stage.discard().unwrap();
}
