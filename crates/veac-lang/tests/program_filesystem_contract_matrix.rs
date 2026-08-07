#![cfg(unix)]

use std::fs;
use std::path::Path;

use tempfile::tempdir;
use veac_lang::program::{prepare_path, FileSystemLoader, SourceLoader};

#[test]
fn filesystem_entry_reports_root_resolution_directory_and_filename_failures() {
    let temp = tempdir().unwrap();
    let missing = temp.path().join("missing/main.veac");
    assert!(FileSystemLoader::for_entry(&missing)
        .unwrap_err()
        .contains("cannot resolve source root"));

    let plain = temp.path().join("plain");
    fs::write(&plain, "module {}").unwrap();
    assert!(FileSystemLoader::for_entry(&plain.join("main.veac"))
        .unwrap_err()
        .contains("escapes the source root"));
    assert!(FileSystemLoader::for_entry(Path::new("/"))
        .unwrap_err()
        .contains("not a regular file"));
}

#[test]
fn filesystem_loader_rejects_missing_symlink_and_non_directory_components() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let module = temp.path().join("module.veac");
    fs::write(&entry, "module {}").unwrap();
    fs::write(&module, "module {}").unwrap();
    let (loader, _) = FileSystemLoader::for_entry(&entry).unwrap();
    assert_eq!(loader.load("/", "module.veac").unwrap().id, "module.veac");
    assert!(loader
        .load("main.veac", "missing.veac")
        .unwrap_err()
        .contains("unavailable"));

    fs::write(temp.path().join("plain"), "module {}").unwrap();
    assert!(loader
        .load("main.veac", "plain/nested.veac")
        .unwrap_err()
        .contains("unavailable"));
    std::os::unix::fs::symlink(&module, temp.path().join("alias.veac")).unwrap();
    assert!(loader
        .load("main.veac", "alias.veac")
        .unwrap_err()
        .contains("unavailable"));
}

#[test]
fn filesystem_snapshot_rejects_non_utf8_and_oversized_entries() {
    let temp = tempdir().unwrap();
    let non_utf8 = temp.path().join("non-utf8.veac");
    fs::write(&non_utf8, [0xff]).unwrap();
    assert_eq!(
        prepare_path(&non_utf8).unwrap_err().as_slice()[0].code,
        "PROGRAM_ENTRY_LOAD"
    );

    let oversized = temp.path().join("oversized.veac");
    fs::File::create(&oversized)
        .unwrap()
        .set_len(16 * 1024 * 1024 + 1)
        .unwrap();
    assert_eq!(
        prepare_path(&oversized).unwrap_err().as_slice()[0].code,
        "PROGRAM_ENTRY_LOAD"
    );
}
