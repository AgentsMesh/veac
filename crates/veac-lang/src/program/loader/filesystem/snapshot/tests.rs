use std::io::Write;
use std::path::{Path, PathBuf};

use rustix::fd::OwnedFd;
use rustix::fs::{open, openat, Mode, OFlags};

use super::{changed_error, inspect_error, read_regular, read_regular_with};

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
fn snapshot_rejects_a_path_removed_after_reading() {
    let (_temp, directory, file, path) = fixture();
    let result = read_regular_with(file, &directory, Path::new("source.veac"), &path, || {
        std::fs::remove_file(&path).unwrap();
    });
    let error = result.err().expect("removed path must invalidate snapshot");
    assert!(error.contains("changed while it was read"));
    assert!(error.contains("source.veac"));
}

#[test]
fn snapshot_io_errors_retain_the_source_label() {
    let label = Path::new("nested/source.veac");
    assert!(inspect_error(label, rustix::io::Errno::IO).contains("nested/source.veac"));
    assert!(changed_error(label, rustix::io::Errno::IO).contains("changed while it was read"));
}

#[test]
fn snapshot_reads_a_stable_utf8_regular_file() {
    let (_temp, directory, file, path) = fixture();
    let opened = read_regular(file, &directory, Path::new("source.veac"), &path).unwrap();
    assert_eq!(opened.source, "original");
}

#[test]
fn snapshot_rejects_non_utf8_and_non_regular_sources() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("source.veac");
    std::fs::write(&path, [0xff]).unwrap();
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
    assert!(
        read_regular(file, &directory, Path::new("source.veac"), &path)
            .err()
            .expect("non-UTF-8 source must fail")
            .contains("not UTF-8")
    );

    let subdirectory = temp.path().join("folder");
    std::fs::create_dir(&subdirectory).unwrap();
    let file = open(
        &subdirectory,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    assert!(
        read_regular_with(file, &directory, Path::new("folder"), &subdirectory, || {})
            .err()
            .expect("directory source must fail")
            .contains("not a regular file")
    );
}

#[test]
fn snapshot_rejects_a_sparse_file_over_the_source_limit() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("large.veac");
    std::fs::File::create(&path)
        .unwrap()
        .set_len((crate::program::limits::MAX_SOURCE_BYTES + 1) as u64)
        .unwrap();
    let directory = open(
        temp.path(),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    let file = openat(
        &directory,
        "large.veac",
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    assert!(
        read_regular(file, &directory, Path::new("large.veac"), &path)
            .err()
            .expect("oversized source must fail")
            .contains("exceeds 16 MiB")
    );
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
