use std::path::Path;

use veac_artifact::ContentDigest;

use super::super::{file_state_until, FileState};
use crate::RuntimeError;

#[test]
fn file_state_hashes_regular_and_explicitly_empty_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let regular = temp.path().join("regular.bin");
    let bytes = vec![7_u8; 70_000];
    std::fs::write(&regular, &bytes).unwrap();
    let state = file_state(&regular, false).unwrap();
    assert_eq!(state.content, ContentDigest::sha256(bytes));
    assert_eq!(state.size_bytes, 70_000);

    let empty = temp.path().join("empty.srt");
    std::fs::write(&empty, []).unwrap();
    assert!(file_state(&empty, false).is_err());
    assert_eq!(file_state(&empty, true).unwrap().size_bytes, 0);
}

fn file_state(path: &Path, allow_empty: bool) -> Result<FileState, RuntimeError> {
    file_state_until(
        path,
        allow_empty,
        std::time::Instant::now() + std::time::Duration::from_secs(10),
    )
}

#[test]
fn file_state_rejects_missing_directories_and_symlinks() {
    let temp = tempfile::tempdir().unwrap();
    assert!(file_state(&temp.path().join("missing"), false)
        .unwrap_err()
        .message
        .contains("unavailable"));
    assert!(file_state(temp.path(), false)
        .unwrap_err()
        .message
        .contains("unavailable or unsafe"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let target = temp.path().join("target");
        let link = temp.path().join("link");
        std::fs::write(&target, b"value").unwrap();
        symlink(target, &link).unwrap();
        assert!(file_state(&link, false).is_err());
    }
}

#[cfg(unix)]
#[test]
fn unreadable_regular_output_reports_open_failure() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("unreadable");
    std::fs::write(&path, b"value").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o0)).unwrap();
    let result = file_state(&path, false);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(result
        .unwrap_err()
        .message
        .contains("unavailable or unsafe"));
}
