use super::{BuildCache, CacheReservation, MemoryBuildCache, NodeCacheKey, NullBuildCache};
use crate::{ArtifactOutputs, BuildErrorKind, CancellationToken, ContentDigest, OutputSlot};

fn key(value: &str) -> NodeCacheKey {
    NodeCacheKey::computation("test", 1, ContentDigest::sha256(value.as_bytes())).unwrap()
}

fn outputs(value: &str) -> ArtifactOutputs {
    ArtifactOutputs::one(
        &OutputSlot::<()>::new("result").unwrap(),
        ContentDigest::sha256(value.as_bytes()),
    )
    .unwrap()
}

#[test]
fn cache_keys_are_ordered_by_digest_and_expose_their_contract() {
    let left = key("left");
    let right = key("right");
    assert_eq!(left.partial_cmp(&right), Some(left.cmp(&right)));
    assert_eq!(left == right, left.digest() == right.digest());
    assert_eq!(left.descriptor().producer.configuration.value.len(), 64);

    let encoded = serde_json::to_value(&left).unwrap();
    let decoded: NodeCacheKey = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, left);
}

#[test]
fn computation_keys_reject_invalid_configuration_digests() {
    let error = NodeCacheKey::computation(
        "invalid",
        1,
        ContentDigest {
            algorithm: veac_artifact::DigestAlgorithm::Sha256,
            value: "invalid".to_owned(),
        },
    )
    .unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::InvalidContract);
    assert!(error.message().contains("invalid computation key"));
}

#[test]
fn default_reservation_distinguishes_cancelled_hit_and_owner() {
    let cache = MemoryBuildCache::new();
    let cache_key = key("cache");
    assert!(cache.is_empty().unwrap());

    let cancelled = CancellationToken::new();
    cancelled.cancel();
    let Err(error) = cache.reserve(&cache_key, &cancelled) else {
        panic!("cancelled reservation unexpectedly succeeded");
    };
    assert_eq!(error.kind(), BuildErrorKind::Cancelled);

    assert!(matches!(
        cache
            .reserve(&cache_key, &CancellationToken::new())
            .unwrap(),
        CacheReservation::Owner(_)
    ));
    cache.put(&cache_key, &outputs("cached")).unwrap();
    assert_eq!(cache.len().unwrap(), 1);
    assert!(matches!(
        cache
            .reserve(&cache_key, &CancellationToken::new())
            .unwrap(),
        CacheReservation::Hit(_)
    ));
}

#[test]
fn null_cache_never_retains_outputs() {
    let cache = NullBuildCache;
    let cache_key = key("null");
    cache.put(&cache_key, &outputs("ignored")).unwrap();
    assert!(cache.get(&cache_key).unwrap().is_none());
    assert!(matches!(
        cache
            .reserve(&cache_key, &CancellationToken::new())
            .unwrap(),
        CacheReservation::Owner(_)
    ));
}
