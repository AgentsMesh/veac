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
fn source_location_rejects_the_final_symlink() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("target.veac");
    std::fs::write(&target, "outside").unwrap();
    let source = temp.path().join("main.veac");
    symlink(&target, &source).unwrap();

    let error = match SourceLocation::resolve(&source) {
        Err(error) => error,
        Ok(_) => panic!("final source symlink must be rejected"),
    };
    assert!(error.to_string().contains("source is not a regular file"));
}
