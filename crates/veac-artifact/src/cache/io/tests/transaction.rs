use std::cell::Cell;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::{artifact_key, test_support, ArtifactErrorKind, ArtifactStore, ContentDigest};

use super::super::authority::target_parent;
use super::super::catalog_scan::catalog_keys;
use super::super::lock::DirectoryLock;

#[test]
fn crash_incomplete_final_key_is_hidden_then_recovered() {
    let fixture = Fixture::new();
    fs::create_dir_all(&fixture.directory).unwrap();
    fs::write(fixture.directory.join("payload.bin"), b"partial").unwrap();

    assert!(fixture.store.get(&fixture.key).unwrap().is_none());
    assert!(catalog_keys(fixture.store.root()).unwrap().is_empty());

    let record = fixture
        .store
        .put(&test_support::descriptor(), b"complete")
        .unwrap();
    assert_eq!(record.key, fixture.key);
    assert_eq!(
        fixture.store.get(&fixture.key).unwrap().unwrap().payload,
        b"complete"
    );
    assert_eq!(entry_names(&fixture.directory), committed_entries());
    assert_eq!(
        fs::metadata(fixture.directory.join("sealed"))
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn recovery_never_unlinks_a_foreign_symlink_or_unexpected_entry() {
    let fixture = Fixture::new();
    fs::create_dir_all(&fixture.directory).unwrap();
    let external = fixture.temp.path().join("external");
    fs::write(&external, b"safe").unwrap();
    symlink(&external, fixture.directory.join("payload.bin")).unwrap();

    assert!(fixture.store.get(&fixture.key).unwrap().is_none());
    let error = fixture
        .store
        .put(&test_support::descriptor(), b"replacement")
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::CorruptCache);
    assert_eq!(fs::read(&external).unwrap(), b"safe");
    assert!(fs::symlink_metadata(fixture.directory.join("payload.bin"))
        .unwrap()
        .file_type()
        .is_symlink());

    fs::remove_file(fixture.directory.join("payload.bin")).unwrap();
    fs::write(fixture.directory.join("foreign"), b"owned elsewhere").unwrap();
    assert_eq!(
        fixture.store.get(&fixture.key).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
    assert_eq!(
        fs::read(fixture.directory.join("foreign")).unwrap(),
        b"owned elsewhere"
    );
}

#[test]
fn remove_recovers_an_incomplete_final_key_as_absent() {
    let fixture = Fixture::new();
    fs::create_dir_all(&fixture.directory).unwrap();
    fs::write(fixture.directory.join("record.json"), b"partial").unwrap();

    assert!(!fixture.store.remove(&fixture.key).unwrap());
    assert!(!fixture.directory.exists());
}

#[test]
fn malformed_commit_markers_are_corruption_not_recovery_candidates() {
    let fixture = Fixture::new();
    fs::create_dir_all(&fixture.directory).unwrap();
    for name in ["descriptor.json", "payload.bin", "record.json"] {
        fs::write(fixture.directory.join(name), []).unwrap();
    }
    fs::write(fixture.directory.join("sealed"), b"not empty").unwrap();

    assert_eq!(
        fixture.store.get(&fixture.key).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
    assert!(fixture.directory.join("sealed").is_file());
    assert_eq!(
        fixture
            .store
            .put(&test_support::descriptor(), b"replacement")
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
}

#[test]
fn catalog_ignores_a_bound_legacy_stage_orphan() {
    let fixture = Fixture::new();
    let record = fixture
        .store
        .put(&test_support::descriptor(), b"payload")
        .unwrap();
    let orphan = fixture
        .directory
        .parent()
        .unwrap()
        .join(".veac-cache-00000000000000000000000000000000");
    fs::create_dir(&orphan).unwrap();
    fs::write(orphan.join("payload.bin"), b"partial").unwrap();

    assert_eq!(catalog_keys(fixture.store.root()).unwrap(), [record.key]);
    assert!(orphan.is_dir());
    fs::write(orphan.join("foreign"), b"unexpected").unwrap();
    assert_eq!(
        catalog_keys(fixture.store.root()).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
}

#[test]
fn prefix_lock_waits_are_guarded() {
    let fixture = Fixture::new();
    let (first, _) = target_parent(fixture.store.root(), &fixture.directory, true)
        .unwrap()
        .unwrap();
    let lock = DirectoryLock::exclusive_while(first.current(), || true).unwrap();
    let (second, _) = target_parent(fixture.store.root(), &fixture.directory, false)
        .unwrap()
        .unwrap();
    let calls = Cell::new(0_u8);
    let error = DirectoryLock::shared_while(second.current(), || {
        calls.set(calls.get() + 1);
        calls.get() < 5
    })
    .err()
    .expect("the waiting lock must honor its guard");
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert_eq!(calls.get(), 5);
    drop(lock);
    DirectoryLock::shared_while(second.current(), || true).unwrap();
}

struct Fixture {
    temp: tempfile::TempDir,
    store: ArtifactStore,
    key: ContentDigest,
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let store = ArtifactStore::new(temp.path().join("store"));
        let key = artifact_key(&test_support::descriptor()).unwrap();
        let directory = cache_directory(store.root(), &key);
        Self {
            temp,
            store,
            key,
            directory,
        }
    }
}

fn cache_directory(root: &Path, key: &ContentDigest) -> PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}

fn entry_names(directory: &Path) -> Vec<String> {
    let mut names = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn committed_entries() -> Vec<String> {
    ["descriptor.json", "payload.bin", "record.json", "sealed"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}
