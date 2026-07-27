use super::*;
use crate::ArtifactErrorKind;

#[test]
fn symlink_targets_must_be_nonempty_relative_names() {
    assert_eq!(
        relative_names(Path::new(".")).unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
    for path in [Path::new("../escape"), Path::new("/absolute")] {
        assert_eq!(
            relative_names(path).unwrap_err().kind,
            ArtifactErrorKind::UnsafePath
        );
    }
    assert_eq!(
        relative_names(Path::new("safe/./nested")).unwrap(),
        [OsString::from("safe"), OsString::from("nested")]
    );
}

#[test]
fn regular_entries_are_not_treated_as_root_symlinks() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("regular"), b"regular").unwrap();
    let parent = File::open(temp.path()).unwrap();
    assert!(resolve(&parent, OsStr::new("regular")).unwrap().is_none());
}

#[test]
fn missing_links_and_removed_bound_links_fail_as_corruption() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let parent = File::open(temp.path()).unwrap();
    assert_eq!(
        resolve(&parent, OsStr::new("missing")).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );

    let path = temp.path().join("linked");
    symlink("target", &path).unwrap();
    let (guard, names) = resolve(&parent, OsStr::new("linked")).unwrap().unwrap();
    assert_eq!(names, [OsString::from("target")]);
    std::fs::remove_file(path).unwrap();
    assert_eq!(
        guard.verify().unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
}
