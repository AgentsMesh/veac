use std::os::unix::fs::PermissionsExt;

use super::*;
use crate::ArtifactErrorKind;

#[test]
fn open_failure_removes_the_identity_bound_created_directory() {
    let temp = tempfile::tempdir().unwrap();
    let parent = super::super::super::open_directory(temp.path()).unwrap();
    let result = create_private_directory_with(&parent, |name| {
        std::fs::set_permissions(
            temp.path().join(name),
            std::fs::Permissions::from_mode(0o000),
        )
        .unwrap();
    });

    assert_eq!(result.unwrap_err().kind, ArtifactErrorKind::UnsafePath);
    assert!(std::fs::read_dir(temp.path()).unwrap().next().is_none());
}

#[test]
fn cleanup_never_removes_a_replacement_created_directory() {
    let temp = tempfile::tempdir().unwrap();
    let parent = super::super::super::open_directory(temp.path()).unwrap();
    let result = create_private_directory_with(&parent, |name| {
        let path = temp.path().join(name);
        std::fs::remove_dir(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("foreign"), b"foreign").unwrap();
    });

    assert_eq!(result.unwrap_err().kind, ArtifactErrorKind::UnsafePath);
    let foreign = std::fs::read_dir(temp.path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("foreign");
    assert_eq!(std::fs::read(foreign).unwrap(), b"foreign");
}

#[test]
fn created_directory_binding_rejects_missing_and_regular_entries() {
    let temp = tempfile::tempdir().unwrap();
    let parent = super::super::super::open_directory(temp.path()).unwrap();
    let missing = CreatedDirectory::new(&parent, "missing".into())
        .err()
        .unwrap();
    assert_eq!(missing.kind, ArtifactErrorKind::UnsafePath);

    std::fs::write(temp.path().join("regular"), b"regular").unwrap();
    let regular = CreatedDirectory::new(&parent, "regular".into())
        .err()
        .unwrap();
    assert_eq!(regular.kind, ArtifactErrorKind::UnsafePath);
}

#[test]
fn cleanup_accepts_an_already_absent_bound_directory() {
    let temp = tempfile::tempdir().unwrap();
    let parent = super::super::super::open_directory(temp.path()).unwrap();
    let name = OsString::from("created");
    std::fs::create_dir(temp.path().join(&name)).unwrap();
    let mut created = CreatedDirectory::new(&parent, name.clone()).unwrap();
    std::fs::remove_dir(temp.path().join(name)).unwrap();

    created.cleanup().unwrap();
    assert!(!created.armed);
}

#[test]
fn cleanup_requires_a_bound_directory_identity() {
    let temp = tempfile::tempdir().unwrap();
    let parent = super::super::super::open_directory(temp.path()).unwrap();
    let mut created = CreatedDirectory {
        parent: &parent,
        name: "unbound".into(),
        identity: None,
        armed: true,
    };
    assert_eq!(
        created.cleanup().unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
    created.armed = false;
}
