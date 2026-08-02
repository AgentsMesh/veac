use super::SourceLocation;

#[test]
fn source_location_canonicalizes_only_the_parent() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "source").unwrap();

    let location = SourceLocation::resolve(&source).unwrap();

    let root = temp.path().canonicalize().unwrap();
    assert_eq!(location.root(), root);
    assert_eq!(location.module(), "main.veac");
    assert_eq!(location.path(), root.join("main.veac"));
}

#[test]
fn source_location_rejects_invalid_names_and_missing_parents() {
    let temp = tempfile::tempdir().unwrap();
    assert!(SourceLocation::resolve(&temp.path().join("bad:name.veac")).is_err());
    assert!(SourceLocation::resolve(&temp.path().join("missing/main.veac")).is_err());
    assert!(SourceLocation::resolve(std::path::Path::new("/")).is_err());
}

#[cfg(unix)]
#[test]
fn source_location_does_not_follow_the_final_symlink() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("target.veac");
    std::fs::write(&target, "outside").unwrap();
    let source = temp.path().join("main.veac");
    symlink(&target, &source).unwrap();

    let location = SourceLocation::resolve(&source).unwrap();

    assert_eq!(
        location.path(),
        temp.path().canonicalize().unwrap().join("main.veac")
    );
}
