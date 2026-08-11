use super::{fingerprint, snapshot, ProjectRoots};
use crate::BuildErrorKind;
use veac_project::ProjectPath;

#[test]
fn roots_snapshot_regular_source_and_material_files() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let material = temp.path().join("material");
    std::fs::create_dir_all(source.join("nested")).unwrap();
    std::fs::create_dir(&material).unwrap();
    std::fs::write(source.join("nested/main.veac"), b"source").unwrap();
    std::fs::write(material.join("asset.bin"), b"asset").unwrap();
    let roots = ProjectRoots::new(&source, &material).unwrap();

    let source = snapshot(&source, &ProjectPath::new("nested/main.veac")).unwrap();
    let material = roots.material(&ProjectPath::new("asset.bin")).unwrap();
    assert_eq!(source.size_bytes, 6);
    assert_eq!(material.size_bytes, 5);
}

#[test]
fn roots_and_snapshots_reject_missing_or_non_regular_authorities() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    std::fs::create_dir(&root).unwrap();
    let file = temp.path().join("file");
    std::fs::write(&file, b"x").unwrap();
    assert!(ProjectRoots::new(&file, &root).is_err());
    assert!(ProjectRoots::new(temp.path().join("missing"), &root).is_err());

    for value in ["../escape", "/absolute", "missing"] {
        assert!(snapshot(&root, &ProjectPath::new(value)).is_err());
    }
    std::fs::create_dir(root.join("directory")).unwrap();
    let error = snapshot(&root, &ProjectPath::new("directory")).unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::InvalidContract);
    assert!(fingerprint(&root.join("missing")).is_err());
}

#[cfg(unix)]
#[test]
fn snapshots_reject_symlink_components() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("value"), b"x").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("link")).unwrap();
    let error = snapshot(&root, &ProjectPath::new("link/value")).unwrap_err();
    assert!(error.message().contains("symlink"));
}
