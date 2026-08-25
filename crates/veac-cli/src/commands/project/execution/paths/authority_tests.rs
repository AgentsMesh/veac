use std::path::Path;

use super::declared;

fn code(error: crate::error::CliError) -> String {
    error.diagnostics()[0].code.clone()
}

#[test]
fn declared_rejects_absolute_and_parent_paths() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    assert_eq!(
        code(declared(root, "/outside", "root").unwrap_err()),
        "PROJECT_PATH_ESCAPE"
    );
    assert_eq!(
        code(declared(root, "../outside", "root").unwrap_err()),
        "PROJECT_PATH_ESCAPE"
    );
}

#[test]
fn declared_accepts_a_missing_tail_without_creating_it() {
    let temp = tempfile::tempdir().unwrap();
    let path = declared(temp.path(), "future/nested", "root").unwrap();
    assert_eq!(path, temp.path().join("future/nested"));
    assert!(!path.exists());
}

#[test]
fn declared_rejects_regular_file_components() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("file"), b"x").unwrap();
    let error = declared(temp.path(), "file/child", "root").unwrap_err();
    assert_eq!(code(error), "NOT_A_DIRECTORY");
}

#[cfg(unix)]
#[test]
fn declared_rejects_symlinks_and_permission_failures() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("target")).unwrap();
    symlink(temp.path().join("target"), temp.path().join("alias")).unwrap();
    let error = declared(temp.path(), "alias/child", "root").unwrap_err();
    assert_eq!(code(error), "PROJECT_PATH_SYMLINK");

    let blocked = temp.path().join("blocked");
    std::fs::create_dir(&blocked).unwrap();
    let original = std::fs::metadata(&blocked).unwrap().permissions();
    let mut denied = original.clone();
    use std::os::unix::fs::PermissionsExt;
    denied.set_mode(0o0);
    std::fs::set_permissions(&blocked, denied).unwrap();
    let error = declared(temp.path(), "blocked/child", "root").unwrap_err();
    std::fs::set_permissions(&blocked, original).unwrap();
    assert_eq!(code(error), "PATH_UNAVAILABLE");
}

#[test]
fn declared_rejects_non_normal_components() {
    let temp = tempfile::tempdir().unwrap();
    let current = Path::new(".").to_str().unwrap();
    assert_eq!(
        code(declared(temp.path(), current, "root").unwrap_err()),
        "PROJECT_PATH_ESCAPE"
    );
}
