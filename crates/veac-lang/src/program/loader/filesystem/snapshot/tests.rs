use std::io::Write;
use std::path::{Path, PathBuf};

use rustix::fd::OwnedFd;
use rustix::fs::{open, openat, Mode, OFlags};

use super::{changed_error, inspect_error, read_regular_with};

#[test]
fn snapshot_rejects_an_append_after_reading() {
    let (temp, directory, file, path) = fixture();
    let result = read_regular_with(file, &directory, Path::new("source.veac"), &path, || {
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b" changed")
            .unwrap();
    });
    let Err(error) = result else {
        panic!("append should invalidate the source snapshot");
    };
    assert!(error.contains("changed while it was read"));
    drop(temp);
}

#[test]
fn snapshot_rejects_a_replaced_path_after_reading() {
    let (temp, directory, file, path) = fixture();
    let moved = temp.path().join("moved.veac");
    let result = read_regular_with(file, &directory, Path::new("source.veac"), &path, || {
        std::fs::rename(&path, &moved).unwrap();
        std::fs::write(&path, b"original").unwrap();
    });
    let Err(error) = result else {
        panic!("path replacement should invalidate the source snapshot");
    };
    assert!(error.contains("changed while it was read"));
}

#[test]
fn snapshot_io_errors_retain_the_source_label() {
    let label = Path::new("nested/source.veac");
    assert!(inspect_error(label, rustix::io::Errno::IO).contains("nested/source.veac"));
    assert!(changed_error(label, rustix::io::Errno::IO).contains("changed while it was read"));
}

fn fixture() -> (tempfile::TempDir, OwnedFd, OwnedFd, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("source.veac");
    std::fs::write(&path, b"original").unwrap();
    let directory = open(
        temp.path(),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    let file = openat(
        &directory,
        "source.veac",
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    (temp, directory, file, path)
}
