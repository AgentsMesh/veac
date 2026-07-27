use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::{artifact_key, test_support, ArtifactErrorKind, ArtifactRecord, ContentDigest};

use super::super::{catalog_scan, file, inspect, operations};

#[test]
fn buffered_write_rejects_a_replaced_bound_chain() {
    let fixture = Fixture::new();
    let external = fixture.temp.path().join("external");
    fs::create_dir(&external).unwrap();
    fs::write(external.join("victim"), b"safe").unwrap();
    let moved = fixture.temp.path().join("moved-prefix");
    let error = operations::write_atomic_while_with(
        &fixture.root,
        &fixture.directory,
        &fixture.descriptor,
        &fixture.record,
        b"payload",
        |prefix| replace_directory(prefix, &moved, &external),
        |_| {},
        |_| {},
        || true,
    )
    .unwrap_err();
    assert_safety_error(error.kind);
    assert_eq!(fs::read(external.join("victim")).unwrap(), b"safe");
    assert!(!external.join("descriptor.json").exists());
}

#[test]
fn buffered_write_never_writes_through_a_replaced_stage() {
    let fixture = Fixture::new();
    let external = fixture.temp.path().join("external-stage");
    fs::create_dir(&external).unwrap();
    fs::write(external.join("victim"), b"safe").unwrap();
    let moved = fixture.temp.path().join("moved-stage");
    let error = operations::write_atomic_while_with(
        &fixture.root,
        &fixture.directory,
        &fixture.descriptor,
        &fixture.record,
        b"payload",
        |_| {},
        |stage| replace_directory(stage, &moved, &external),
        |_| {},
        || true,
    )
    .unwrap_err();
    assert_safety_error(error.kind);
    assert_eq!(fs::read(external.join("victim")).unwrap(), b"safe");
    assert_eq!(fs::read(moved.join("payload.bin")).unwrap(), b"payload");
}

#[test]
fn streaming_write_revalidates_the_stage_before_publish() {
    let fixture = Fixture::new();
    let source = fixture.temp.path().join("source");
    fs::write(&source, b"payload").unwrap();
    let external = fixture.temp.path().join("external-publish");
    fs::create_dir(&external).unwrap();
    fs::write(external.join("victim"), b"safe").unwrap();
    let moved = fixture.temp.path().join("moved-publish");
    let error = file::write_file_atomic_while_with(
        &fixture.root,
        &fixture.directory,
        &fixture.descriptor,
        &fixture.record,
        &source,
        |_| {},
        |stage| replace_directory(stage, &moved, &external),
        |_| {},
        || true,
    )
    .unwrap_err();
    assert_safety_error(error.kind);
    assert_eq!(fs::read(external.join("victim")).unwrap(), b"safe");
    assert!(fixture.directory.is_symlink());
    assert_eq!(fs::read(moved.join("payload.bin")).unwrap(), b"payload");
}

#[test]
fn read_rejects_an_artifact_directory_replaced_after_entry_open() {
    let fixture = Fixture::stored();
    let external = fixture.temp.path().join("external-read");
    fs::create_dir(&external).unwrap();
    let moved = fixture.temp.path().join("moved-read");
    let error = inspect::inspect_bounded_while_with(
        &fixture.root,
        &fixture.directory,
        crate::MAX_ARTIFACT_METADATA_BYTES,
        |directory| replace_directory(directory, &moved, &external),
        || true,
    )
    .err()
    .expect("replacement must fail closed");
    assert_safety_error(error.kind);
    assert!(moved.join("payload.bin").is_file());
}

#[test]
fn remove_rejects_a_replacement_without_touching_external_files() {
    let fixture = Fixture::stored();
    let external = fixture.temp.path().join("external-remove");
    fs::create_dir(&external).unwrap();
    fs::write(external.join("victim"), b"safe").unwrap();
    let moved = fixture.temp.path().join("moved-remove");
    let error = operations::remove_while_with(
        &fixture.root,
        &fixture.directory,
        |directory| replace_directory(directory, &moved, &external),
        || true,
    )
    .unwrap_err();
    assert_safety_error(error.kind);
    assert_eq!(fs::read(external.join("victim")).unwrap(), b"safe");
    assert!(moved.join("payload.bin").is_file());
}

#[test]
fn catalog_rejects_a_replaced_digest_directory() {
    let fixture = Fixture::stored();
    let external = fixture.temp.path().join("external-catalog");
    fs::create_dir(&external).unwrap();
    fs::write(external.join("victim"), b"safe").unwrap();
    let moved = fixture.temp.path().join("moved-catalog");
    let error = catalog_scan::catalog_keys_with(&fixture.root, |digest| {
        replace_directory(digest, &moved, &external);
    })
    .unwrap_err();
    assert_safety_error(error.kind);
    assert_eq!(fs::read(external.join("victim")).unwrap(), b"safe");
    assert!(moved.is_dir());
}

struct Fixture {
    temp: tempfile::TempDir,
    root: PathBuf,
    directory: PathBuf,
    descriptor: crate::ArtifactDescriptor,
    record: ArtifactRecord,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("store");
        let descriptor = test_support::descriptor();
        let key = artifact_key(&descriptor).unwrap();
        let directory = root
            .join("sha256")
            .join(&key.value[..2])
            .join(&key.value[2..]);
        let record = ArtifactRecord {
            key,
            content: ContentDigest::sha256(b"payload"),
            size_bytes: 7,
        };
        Self {
            temp,
            root,
            directory,
            descriptor,
            record,
        }
    }

    fn stored() -> Self {
        let fixture = Self::new();
        operations::write_atomic_while_with(
            &fixture.root,
            &fixture.directory,
            &fixture.descriptor,
            &fixture.record,
            b"payload",
            |_| {},
            |_| {},
            |_| {},
            || true,
        )
        .unwrap();
        fixture
    }
}

fn replace_directory(original: &Path, moved: &Path, external: &Path) {
    fs::rename(original, moved).unwrap();
    symlink(external, original).unwrap();
}

fn assert_safety_error(kind: ArtifactErrorKind) {
    assert!(matches!(
        kind,
        ArtifactErrorKind::CorruptCache | ArtifactErrorKind::UnsafePath
    ));
}
