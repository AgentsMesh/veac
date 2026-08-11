use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::{
    ArtifactOutputs, BuildCache, ContentDigest, MemoryBuildCache, NodeCacheKey, OutputSlot,
};

struct Payload;

#[test]
fn poisoned_memory_cache_reports_each_operation() {
    let cache = MemoryBuildCache::new();
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = cache.entries.lock().unwrap();
        panic!("poison cache");
    }));
    let key = NodeCacheKey::computation("test", 1, ContentDigest::sha256(b"key")).unwrap();
    let outputs = ArtifactOutputs::one(
        &OutputSlot::<Payload>::new("out").unwrap(),
        ContentDigest::sha256(b"value"),
    )
    .unwrap();

    assert!(cache.len().is_err());
    assert!(cache.get(&key).is_err());
    assert!(cache.put(&key, &outputs).is_err());
}
