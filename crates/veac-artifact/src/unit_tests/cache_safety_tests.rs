use std::fs;

use crate::{test_support::descriptor, *};

#[test]
fn verified_file_rejects_digest_size_and_key_mismatches() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"actual").unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let key = artifact_key(&descriptor()).unwrap();
    for declared in [
        ArtifactRecord {
            key: key.clone(),
            content: ContentDigest::sha256(b"wrong"),
            size_bytes: 6,
        },
        ArtifactRecord {
            key: key.clone(),
            content: ContentDigest::sha256(b"actual"),
            size_bytes: 99,
        },
        ArtifactRecord {
            key: ContentDigest::sha256(b"wrong-key"),
            content: ContentDigest::sha256(b"actual"),
            size_bytes: 6,
        },
    ] {
        assert_eq!(
            store
                .put_verified_file(&descriptor(), &declared, &payload)
                .unwrap_err()
                .kind,
            ArtifactErrorKind::IdentityMismatch
        );
    }
    assert!(store.get(&key).unwrap().is_none());
}

#[test]
fn cache_rejects_unexpected_entries_without_removing_them() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store.put(&descriptor(), b"payload").unwrap();
    let directory = cache_directory(temp.path(), &record.key);
    fs::create_dir(directory.join("unexpected")).unwrap();
    assert_eq!(
        store.get(&record.key).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
    assert!(directory.join("unexpected").is_dir());
}

#[cfg(unix)]
#[test]
fn cache_rejects_symlink_roots_components_and_payloads() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let real = temp.path().join("real");
    fs::create_dir(&real).unwrap();
    let linked_root = temp.path().join("linked");
    symlink(&real, &linked_root).unwrap();
    assert_eq!(
        ArtifactStore::new(&linked_root)
            .put(&descriptor(), b"payload")
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );

    let component_root = temp.path().join("component-store");
    let component_target = temp.path().join("component-target");
    fs::create_dir(&component_root).unwrap();
    fs::create_dir(&component_target).unwrap();
    symlink(&component_target, component_root.join("sha256")).unwrap();
    assert_eq!(
        ArtifactStore::new(&component_root)
            .put(&descriptor(), b"payload")
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );

    let store = ArtifactStore::new(temp.path().join("store"));
    let record = store.put(&descriptor(), b"payload").unwrap();
    let directory = cache_directory(&temp.path().join("store"), &record.key);
    let payload = directory.join("payload.bin");
    fs::remove_file(&payload).unwrap();
    symlink(temp.path().join("external"), &payload).unwrap();
    assert_eq!(
        store.get(&record.key).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );

    fs::remove_dir_all(temp.path().join("store")).unwrap();
    fs::create_dir(temp.path().join("target")).unwrap();
    symlink(temp.path().join("target"), temp.path().join("store")).unwrap();
    assert_eq!(
        store.get(&record.key).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
}

#[cfg(unix)]
#[test]
fn failed_atomic_write_leaves_no_target_or_staging_directory() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("readonly-store");
    fs::create_dir(&root).unwrap();
    let mut permissions = fs::metadata(&root).unwrap().permissions();
    permissions.set_mode(0o500);
    fs::set_permissions(&root, permissions).unwrap();
    let result = ArtifactStore::new(&root).put(&descriptor(), b"payload");
    let mut permissions = fs::metadata(&root).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&root, permissions).unwrap();
    assert!(result.is_err());
    let key = artifact_key(&descriptor()).unwrap();
    assert!(!cache_directory(&root, &key).exists());
    assert!(fs::read_dir(&root).unwrap().all(|entry| !entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".tmp-")));
}

#[cfg(unix)]
#[test]
fn verified_file_rejects_symlink_payload() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let actual = temp.path().join("actual");
    let linked = temp.path().join("linked");
    fs::write(&actual, b"payload").unwrap();
    symlink(&actual, &linked).unwrap();
    let record = ArtifactRecord {
        key: artifact_key(&descriptor()).unwrap(),
        content: ContentDigest::sha256(b"payload"),
        size_bytes: 7,
    };
    assert_eq!(
        ArtifactStore::new(temp.path().join("store"))
            .put_verified_file(&descriptor(), &record, &linked)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::UnsafePath
    );
}

fn cache_directory(root: &std::path::Path, key: &ContentDigest) -> std::path::PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}
