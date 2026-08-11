use std::path::Path;

use rustix::fs::{open, Mode, OFlags};
use tempfile::tempdir;

use super::{open_source, FileSystemLoader};
use crate::program::loader::SourceLoader;

fn root() -> (tempfile::TempDir, rustix::fd::OwnedFd) {
    let temp = tempdir().unwrap();
    let directory = open(
        temp.path(),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    (temp, directory)
}

#[test]
fn open_source_rejects_empty_and_non_normal_components() {
    let (_temp, directory) = root();
    let empty = open_source(&directory, Path::new(""), Path::new("empty"));
    assert!(empty
        .err()
        .expect("empty path must fail")
        .contains("empty path"));
    let escaped = open_source(
        &directory,
        Path::new("../outside.veac"),
        Path::new("outside.veac"),
    );
    assert!(escaped
        .err()
        .expect("parent component must fail")
        .contains("escapes"));
}

#[test]
fn loader_rejects_directories_and_files_used_as_directories() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    std::fs::write(&entry, "fn main(context: Context) -> Project { context }").unwrap();
    std::fs::create_dir(temp.path().join("directory.veac")).unwrap();
    std::fs::write(temp.path().join("plain"), "module {}").unwrap();
    let (loader, _) = FileSystemLoader::for_entry(&entry).unwrap();
    assert!(loader
        .load("main.veac", "directory.veac")
        .unwrap_err()
        .contains("not a regular file"));
    assert!(loader
        .load("main.veac", "plain/module.veac")
        .unwrap_err()
        .contains("unavailable"));
}

#[test]
fn entry_reports_missing_parent_and_missing_file_name() {
    let temp = tempdir().unwrap();
    let missing = temp.path().join("missing/main.veac");
    assert!(FileSystemLoader::for_entry(&missing)
        .unwrap_err()
        .contains("cannot resolve source root"));
    assert!(FileSystemLoader::for_entry(Path::new("/"))
        .unwrap_err()
        .contains("not a regular file"));
}

#[test]
fn explicit_root_supports_nested_entries_and_rejects_escape() {
    let temp = tempdir().unwrap();
    std::fs::create_dir(temp.path().join("nested")).unwrap();
    std::fs::write(
        temp.path().join("nested/project.veac"),
        "fn project() -> int { 1 }",
    )
    .unwrap();
    let (loader, loaded) =
        FileSystemLoader::for_root_entry(temp.path(), Path::new("nested/project.veac")).unwrap();
    assert_eq!(loader.root(), temp.path().canonicalize().unwrap());
    assert_eq!(loaded.id, "nested/project.veac");
    assert!(
        FileSystemLoader::for_root_entry(temp.path(), Path::new("../outside.veac"))
            .unwrap_err()
            .contains("root-confined")
    );
}

#[test]
fn loader_rejects_two_ids_for_one_physical_file() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let original = temp.path().join("original.veac");
    std::fs::write(&entry, "fn main(context: Context) -> Project { context }").unwrap();
    std::fs::write(&original, "module {}").unwrap();
    std::fs::hard_link(&original, temp.path().join("alias.veac")).unwrap();
    let (loader, _) = FileSystemLoader::for_entry(&entry).unwrap();
    loader.load("main.veac", "original.veac").unwrap();
    assert!(loader
        .load("main.veac", "alias.veac")
        .unwrap_err()
        .contains("same physical file"));
}
