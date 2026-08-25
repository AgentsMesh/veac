use super::source_loader_for_location;

#[test]
fn loader_uses_the_captured_location_after_a_parent_symlink_retarget() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    std::fs::create_dir(&first).unwrap();
    std::fs::create_dir(&second).unwrap();
    std::fs::write(
        first.join("main.veac"),
        "module { export const value = 1; }\n",
    )
    .unwrap();
    std::fs::write(
        second.join("main.veac"),
        "module { export const value = 2; }\n",
    )
    .unwrap();
    let alias = temp.path().join("project");
    std::os::unix::fs::symlink(&first, &alias).unwrap();
    let requested = alias.join("main.veac");
    let location = crate::fs::SourceLocation::resolve(&requested).unwrap();

    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&second, &alias).unwrap();
    let snapshot = source_loader_for_location(&location, &[]).unwrap();

    assert!(snapshot.entry.source.contains("value = 1"));
    assert!(!snapshot.entry.source.contains("value = 2"));
}
