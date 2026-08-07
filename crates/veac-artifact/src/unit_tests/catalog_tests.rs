use std::fs;

use crate::*;

#[path = "catalog_tests/support.rs"]
mod support;

use support::{cache_directory, dependency, descriptor};

#[test]
fn catalog_matches_exact_dependencies_and_sorts_by_key() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let source = dependency(ArtifactDependencyRole::Source, b"source");
    let profile = dependency(ArtifactDependencyRole::Profile, b"preview");
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
        .all(|artifact| artifact.descriptor().kind() == ArtifactKind::ProxyVideo));

    let narrow = ArtifactCatalogQuery::new(ArtifactKind::ProxyVideo, vec![profile]).unwrap();
    let matches = store.catalog(&narrow).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].record(), &records[0]);
}

#[test]
fn catalog_never_wildcards_dependency_roles_or_identities() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let source = dependency(ArtifactDependencyRole::Source, b"source");
    store
        .put(
            &descriptor(ArtifactKind::RenderSegment, vec![source], 1),
            b"segment",
        )
        .unwrap();
    for dependency in [
        dependency(ArtifactDependencyRole::Input, b"source"),
        dependency(ArtifactDependencyRole::Source, b"other"),
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
    let invalid_digest = ArtifactDependency {
        role: ArtifactDependencyRole::Source,
        identity: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: "bad".into(),
        },
    };
    for dependencies in [
        vec![invalid_digest],
        vec![dependency(ArtifactDependencyRole::Source, b"source"); 2],
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
        vec![dependency(ArtifactDependencyRole::Source, b"source")],
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
    let source = dependency(ArtifactDependencyRole::Source, b"source");
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
        vec![dependency(ArtifactDependencyRole::Source, b"source")],
    )
    .unwrap();
    assert_eq!(
        ArtifactStore::new(root).catalog(&query).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
}
