#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tempfile::tempdir;
use veac_lang::program::{prepare_path, FileSystemLoader, SourceLoader};

use super::executable_entry;

fn fifo(path: &Path) {
    assert!(Command::new("mkfifo").arg(path).status().unwrap().success());
}

fn compile_quickly(path: PathBuf) -> (String, String) {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::spawn(move || {
        let error = prepare_path(&path).unwrap_err();
        let diagnostic = &error.as_slice()[0];
        sender
            .send((diagnostic.code.to_owned(), diagnostic.message.clone()))
            .unwrap();
    });
    let result = receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("source loading blocked on a special file");
    handle.join().unwrap();
    result
}

#[test]
fn filesystem_entry_fifo_is_rejected_without_blocking() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fifo(&entry);
    let (code, message) = compile_quickly(entry);
    assert_eq!(code, "PROGRAM_ENTRY_LOAD");
    assert!(message.contains("not a regular file"));
}

#[test]
fn filesystem_import_fifo_is_rejected_without_blocking() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let module = temp.path().join("module.veac");
    fs::write(
        &entry,
        executable_entry("import \"./module.veac\" as module;", "1s"),
    )
    .unwrap();
    fifo(&module);
    let (code, message) = compile_quickly(entry);
    assert_eq!(code, "PROGRAM_IMPORT_LOAD");
    assert!(message.contains("not a regular file"));
}

#[test]
fn filesystem_compile_rejects_hard_link_module_aliases() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let first = temp.path().join("first.veac");
    let alias = temp.path().join("alias.veac");
    fs::write(
        &entry,
        executable_entry(
            "import \"./first.veac\" as first;\nimport \"./alias.veac\" as alias;",
            "1s",
        ),
    )
    .unwrap();
    fs::write(&first, "module {}").unwrap();
    fs::hard_link(&first, &alias).unwrap();
    let error = prepare_path(&entry).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_IMPORT_LOAD");
    assert!(error.as_slice()[0].message.contains("same physical file"));
}

#[test]
fn filesystem_loader_clones_share_physical_identity_state() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let first = temp.path().join("first.veac");
    let alias = temp.path().join("alias.veac");
    fs::write(&entry, executable_entry("", "1s")).unwrap();
    fs::write(&first, "module {}").unwrap();
    fs::hard_link(&first, &alias).unwrap();
    let (loader, _) = FileSystemLoader::for_entry(&entry).unwrap();
    assert_eq!(loader.root(), temp.path().canonicalize().unwrap());
    let cloned = loader.clone();
    loader.load("main.veac", "first.veac").unwrap();
    assert!(cloned
        .load("main.veac", "alias.veac")
        .unwrap_err()
        .contains("same physical file"));
}
