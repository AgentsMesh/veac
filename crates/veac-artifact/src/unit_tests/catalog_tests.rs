use std::fs;

use serde_json::json;

use crate::{test_support, *};

#[test]
fn catalog_matches_exact_dependencies_and_sorts_by_key() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let source = dependency("source", b"source");
    let profile = dependency("profile", b"preview");
    let first = descriptor(ArtifactKind::ProxyVideo, vec![source.clone()], 1);
    let second = descriptor(
        ArtifactKind::ProxyVideo,
        vec![profile.clone(), source.clone()],
        2,
    );
    let other_kind = descriptor(ArtifactKind::ProxyAudio, vec![source.clone()], 3);
    let records = [
        store.put(&second, b"second").unwrap(),
        store.put(&other_kind, b"audio").unwrap(),
        store.put(&first, b"first").unwrap(),
    ];

    let query = ArtifactCatalogQuery::new(ArtifactKind::ProxyVideo, vec![source]).unwrap();
    assert_eq!(query.kind(), ArtifactKind::ProxyVideo);
    assert_eq!(query.dependencies().len(), 1);
    let matches = store.catalog(&query).unwrap();
    assert_eq!(matches.len(), 2);
    assert!(matches[0].record().key.value < matches[1].record().key.value);
    assert!(matches
        .iter()
        .all(|artifact| artifact.descriptor().kind == ArtifactKind::ProxyVideo));

    let narrow = ArtifactCatalogQuery::new(ArtifactKind::ProxyVideo, vec![profile]).unwrap();
    let matches = store.catalog(&narrow).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].record(), &records[0]);
}

#[test]
fn catalog_never_wildcards_dependency_roles_or_identities() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let source = dependency("source", b"source");
    store
        .put(
            &descriptor(ArtifactKind::RenderSegment, vec![source], 1),
            b"segment",
        )
        .unwrap();
    for dependency in [
        dependency("input", b"source"),
        dependency("source", b"other"),
    ] {
        let query =
            ArtifactCatalogQuery::new(ArtifactKind::RenderSegment, vec![dependency]).unwrap();
        assert!(store.catalog(&query).unwrap().is_empty());
    }
}

#[test]
fn catalog_query_rejects_empty_invalid_and_duplicate_dependencies() {
    assert_eq!(
        ArtifactCatalogQuery::new(ArtifactKind::ProxyVideo, vec![])
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );
    let mut empty_role = dependency("source", b"source");
    empty_role.role.clear();
    let invalid_digest = ArtifactDependency {
        role: "source".into(),
        identity: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: "bad".into(),
        },
    };
    for dependencies in [
        vec![empty_role],
        vec![invalid_digest],
        vec![dependency("source", b"source"); 2],
    ] {
        assert_eq!(
            ArtifactCatalogQuery::new(ArtifactKind::ProxyVideo, dependencies)
                .unwrap_err()
                .kind,
            ArtifactErrorKind::InvalidContract
        );
    }
}

#[test]
fn catalog_is_empty_for_absent_or_initialized_empty_stores() {
    let temp = tempfile::tempdir().unwrap();
    let query = ArtifactCatalogQuery::new(
        ArtifactKind::ProxyVideo,
        vec![dependency("source", b"source")],
    )
    .unwrap();
    assert!(ArtifactStore::new(temp.path().join("absent"))
        .catalog(&query)
        .unwrap()
        .is_empty());
    assert!(ArtifactStore::new(temp.path())
        .catalog(&query)
        .unwrap()
        .is_empty());
}

#[test]
fn catalog_fails_closed_on_unexpected_or_corrupt_entries() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let source = dependency("source", b"source");
    let record = store
        .put(
            &descriptor(ArtifactKind::ProxyAudio, vec![source.clone()], 1),
            b"audio",
        )
        .unwrap();
    let query = ArtifactCatalogQuery::new(ArtifactKind::ProxyVideo, vec![source]).unwrap();
    fs::write(
        cache_directory(store.root(), &record.key).join("payload.bin"),
        b"corrupt",
    )
    .unwrap();
    assert_eq!(
        store.catalog(&query).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );

    fs::remove_dir_all(store.root()).unwrap();
    fs::create_dir(store.root()).unwrap();
    fs::write(store.root().join("unexpected"), b"entry").unwrap();
    assert_eq!(
        store.catalog(&query).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
}

#[cfg(unix)]
#[test]
fn catalog_rejects_symlinked_digest_directories() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    fs::create_dir(&root).unwrap();
    symlink(temp.path(), root.join("sha256")).unwrap();
    let query = ArtifactCatalogQuery::new(
        ArtifactKind::ProxyVideo,
        vec![dependency("source", b"source")],
    )
    .unwrap();
    assert_eq!(
        ArtifactStore::new(root).catalog(&query).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
}

fn descriptor(
    kind: ArtifactKind,
    mut dependencies: Vec<ArtifactDependency>,
    tag: u8,
) -> ArtifactDescriptor {
    dependencies.sort_by(|left, right| {
        (&left.role, &left.identity.value).cmp(&(&right.role, &right.identity.value))
    });
    ArtifactDescriptor::new(
        kind,
        test_support::producer(),
        dependencies,
        json!({"tag": tag}),
    )
}

fn dependency(role: &str, bytes: &[u8]) -> ArtifactDependency {
    ArtifactDependency {
        role: role.into(),
        identity: ContentDigest::sha256(bytes),
    }
}

fn cache_directory(root: &std::path::Path, key: &ContentDigest) -> std::path::PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}
