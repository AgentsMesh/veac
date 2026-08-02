use std::collections::BTreeMap;
use std::fs;

use tempfile::tempdir;

use super::super::loader::{normalize, FileSystemLoader, MemoryLoader, SourceLoader};

#[test]
fn memory_loader_confines_every_request_to_the_source_root() {
    let loader = MemoryLoader::new(BTreeMap::from([
        ("module.veac".to_owned(), "module {}".to_owned()),
        ("dir/module.veac".to_owned(), "module {}".to_owned()),
    ]));
    assert_eq!(
        loader.load("main.veac", "./module.veac").unwrap().id,
        "module.veac"
    );
    assert_eq!(
        loader.load("dir/main.veac", "module.veac").unwrap().id,
        "dir/module.veac"
    );
    for invalid in [
        "",
        "/module.veac",
        "../module.veac",
        "dir/../module.veac",
        r"dir\module.veac",
        "dir:module.veac",
        "dir/\u{7f}module.veac",
        "dir/\u{85}module.veac",
    ] {
        assert!(loader
            .load("main.veac", invalid)
            .unwrap_err()
            .contains("root-confined"));
    }
    assert!(loader
        .load("main.veac", &"a".repeat(4097))
        .unwrap_err()
        .contains("root-confined"));
}

#[test]
fn filesystem_loader_resolves_normal_relative_modules() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::create_dir(temp.path().join("nested")).unwrap();
    fs::write(&entry, "project p {}").unwrap();
    fs::write(temp.path().join("nested/module.veac"), "module {}").unwrap();
    let (loader, _) = FileSystemLoader::for_entry(&entry).unwrap();
    let loaded = loader.load("main.veac", "nested/module.veac").unwrap();
    assert_eq!(loaded.id, "nested/module.veac");
}

#[cfg(unix)]
#[test]
fn filesystem_loader_rejects_symlinks_that_escape_the_root() {
    use std::os::unix::fs::symlink;

    let root = tempdir().unwrap();
    let outside = tempdir().unwrap();
    let entry = root.path().join("main.veac");
    fs::write(&entry, "project p {}").unwrap();
    fs::write(outside.path().join("outside.veac"), "module {}").unwrap();
    symlink(
        outside.path().join("outside.veac"),
        root.path().join("linked.veac"),
    )
    .unwrap();
    let (loader, _) = FileSystemLoader::for_entry(&entry).unwrap();
    assert!(loader
        .load("main.veac", "linked.veac")
        .unwrap_err()
        .contains("escapes"));

    symlink(outside.path(), root.path().join("linked-directory")).unwrap();
    assert!(loader
        .load("main.veac", "linked-directory/outside.veac")
        .unwrap_err()
        .contains("symlink"));
}

#[cfg(unix)]
#[test]
fn filesystem_loader_rejects_a_symlink_entry() {
    use std::os::unix::fs::symlink;

    let root = tempdir().unwrap();
    let target = root.path().join("target.veac");
    let entry = root.path().join("main.veac");
    fs::write(&target, "project p {}").unwrap();
    symlink(&target, &entry).unwrap();
    assert!(FileSystemLoader::for_entry(&entry)
        .unwrap_err()
        .contains("symlink"));
}

#[cfg(unix)]
#[test]
fn filesystem_source_ids_reject_non_utf8_paths() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let invalid = std::path::PathBuf::from(OsString::from_vec(b"module-\xff.veac".to_vec()));
    assert!(normalize(&invalid).unwrap_err().contains("UTF-8"));
}
