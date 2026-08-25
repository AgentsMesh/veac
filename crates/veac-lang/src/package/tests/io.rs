use std::path::Path;

use rustix::fs::{open, openat, Mode, OFlags};

use super::*;

#[test]
fn bound_directory_reads_only_stable_regular_bounded_files() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("nested")).unwrap();
    std::fs::write(temp.path().join("nested/value"), b"value").unwrap();
    let root = fs::BoundDirectory::open(temp.path()).unwrap();
    assert_eq!(root.read("nested/value", 5).unwrap(), b"value");
    assert_eq!(
        root.directory("nested").unwrap().read("value", 5).unwrap(),
        b"value"
    );
    assert!(root.read("nested", 100).is_err());
    assert!(root.read("nested/value", 4).is_err());
    assert!(root.read("../value", 100).is_err());
    assert!(fs::BoundDirectory::open(&temp.path().join("missing")).is_err());
}

#[test]
fn bound_text_requires_utf8_and_rejects_symlinks() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("binary"), [0xff]).unwrap();
    let root = fs::BoundDirectory::open(temp.path()).unwrap();
    assert_eq!(
        root.read_text("binary", 1).unwrap_err().kind(),
        PackageErrorKind::Json
    );

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(Path::new("binary"), temp.path().join("alias")).unwrap();
        assert_eq!(
            root.read("alias", 1).unwrap_err().kind(),
            PackageErrorKind::RootEscape
        );
    }
}

#[test]
fn stable_read_rejects_path_replacement_after_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("value");
    std::fs::write(&path, b"value").unwrap();
    let directory = open(
        temp.path(),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    let file = openat(
        &directory,
        "value",
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    let moved = temp.path().join("moved");
    let result = fs::read_stable_with(file, &directory, Path::new("value"), "value", 5, || {
        std::fs::rename(&path, moved).unwrap();
        std::fs::write(&path, b"value").unwrap();
    });
    assert!(result
        .unwrap_err()
        .message()
        .contains("changed while it was read"));
}
