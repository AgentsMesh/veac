use super::*;
use crate::ArtifactErrorKind;

#[test]
fn directory_enumeration_enforces_its_entry_limit() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("a"), b"a").unwrap();
    std::fs::write(temp.path().join("b"), b"b").unwrap();
    let directory = File::open(temp.path()).unwrap();
    assert_eq!(
        directory_entries(&directory, 1).unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );
}

#[test]
fn bound_directory_open_and_create_reject_regular_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    std::fs::create_dir(&root).unwrap();
    let target = root.join("entry");
    std::fs::write(&target, b"file").unwrap();

    let (parent, name) = super::super::authority::target_parent(&root, &target, false)
        .unwrap()
        .unwrap();
    assert_eq!(
        BoundDirectory::open(parent, name).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
    let (parent, name) = super::super::authority::target_parent(&root, &target, false)
        .unwrap()
        .unwrap();
    assert_eq!(
        BoundDirectory::create(parent, name).unwrap_err().kind,
        ArtifactErrorKind::Io
    );
}

#[test]
fn bound_directory_reopen_and_nonempty_removal_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    std::fs::create_dir(&root).unwrap();
    let target = root.join("entry");
    std::fs::create_dir(&target).unwrap();
    let (parent, name) = super::super::authority::target_parent(&root, &target, false)
        .unwrap()
        .unwrap();
    let bound = BoundDirectory::open(parent, name).unwrap().unwrap();
    std::fs::write(target.join("child"), b"child").unwrap();
    assert_eq!(
        bound.remove_empty().unwrap_err().kind,
        ArtifactErrorKind::Io
    );

    std::fs::remove_file(target.join("child")).unwrap();
    let moved = root.join("moved");
    std::fs::rename(&target, &moved).unwrap();
    assert_eq!(
        bound.verify().unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
}
