use std::sync::Arc;

use super::{entry_overhead, BoundedCache};

#[test]
fn accounting_overflow_bypasses_with_an_unbounded_byte_limit() {
    let mut cache = BoundedCache::new(usize::MAX, usize::MAX);
    let outcome = cache.insert(1_u8, usize::MAX, Arc::new(()));

    assert!(outcome.bypassed);
    assert!(!outcome.inserted);
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.retained_bytes(), 0);
}

#[test]
fn retained_total_overflow_bypasses_instead_of_evicting() {
    let mut cache = BoundedCache::new(usize::MAX, usize::MAX);
    assert!(cache.insert(1_u8, 0, Arc::new(())).inserted);
    let retained = cache.retained_bytes();
    let overhead = entry_overhead::<u8, ()>().unwrap();
    let charged = usize::MAX - retained + 1;
    let outcome = cache.insert(2, charged - overhead, Arc::new(()));

    assert!(outcome.bypassed);
    assert_eq!(outcome.evictions, 0);
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.retained_bytes(), retained);
}

#[test]
fn clear_releases_fifo_backing_allocation() {
    let mut cache = BoundedCache::new(128, usize::MAX);
    for key in 0_u16..128 {
        assert!(cache.insert(key, 0, Arc::new(())).inserted);
    }
    assert!(cache.order.capacity() > 0);

    cache.clear();

    assert_eq!(cache.order.capacity(), 0);
    assert!(cache.values.is_empty());
    assert_eq!(cache.retained_bytes(), 0);
}
