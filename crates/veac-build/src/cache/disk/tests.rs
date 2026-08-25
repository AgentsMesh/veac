use super::DiskBuildCache;
use crate::{BuildCache, BuildErrorKind, CancellationToken, ContentDigest, NodeCacheKey};
use veac_artifact::ArtifactStore;

fn key() -> NodeCacheKey {
    NodeCacheKey::computation("disk-test", 1, ContentDigest::sha256(b"configuration")).unwrap()
}

#[test]
fn disk_cache_maps_filesystem_failures_and_wait_cancellation() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("not-a-directory");
    std::fs::write(&file, b"x").unwrap();
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    assert!(DiskBuildCache::new(store.clone(), file.join("leases")).is_err());

    let cache = DiskBuildCache::new(store, temp.path().join("leases")).unwrap();
    let token = CancellationToken::new();
    token.cancel();
    let Err(error) = cache.reserve(&key(), &token) else {
        panic!("cancelled reservation unexpectedly succeeded");
    };
    assert_eq!(error.kind(), BuildErrorKind::Cancelled);
}

#[test]
fn disk_cache_maps_artifact_store_failures() {
    let temp = tempfile::tempdir().unwrap();
    let store_root = temp.path().join("store-file");
    std::fs::write(&store_root, b"x").unwrap();
    let cache =
        DiskBuildCache::new(ArtifactStore::new(&store_root), temp.path().join("leases")).unwrap();
    let error = cache.get(&key()).unwrap_err();
    assert!(error.message().contains("artifact cache failure"));
}

#[test]
fn disk_cache_rejects_a_non_json_computation_record() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    let cache = DiskBuildCache::new(store, temp.path().join("leases")).unwrap();
    let cache_key = key();
    cache
        .artifact_store()
        .put(cache_key.descriptor(), b"not-json")
        .unwrap();
    let error = cache.get(&cache_key).unwrap_err();
    assert!(error.message().contains("invalid computation record"));
}
