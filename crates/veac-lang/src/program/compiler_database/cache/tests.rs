use std::sync::Arc;

use super::{entry_overhead, SyntaxCache};
use crate::program::compiler_database::key::SyntaxQueryKey;
use crate::program::compiler_database::CompilerDatabaseLimits;

const SOURCE: &str =
    "fn main(context: Context) -> Project { project(identifier(\"p\"), project_settings(1)) }";

#[test]
fn accounting_overflow_bypasses_with_an_unbounded_byte_limit() {
    let mut cache = unbounded_cache();
    let key = SyntaxQueryKey::new("entry.veac", SOURCE);
    cache.statistics.syntax_bypasses = u64::MAX;
    cache.insert(key, usize::MAX, surface("entry.veac"));

    assert_eq!(cache.statistics.syntax_bypasses, u64::MAX);
    assert!(cache.values.is_empty());
    assert_eq!(cache.statistics.cached_syntax_source_bytes, 0);
}

#[test]
fn retained_total_overflow_bypasses_without_eviction() {
    let mut cache = unbounded_cache();
    let first = SyntaxQueryKey::new("first.veac", SOURCE);
    cache.insert(first, 0, surface("first.veac"));
    let retained = cache.statistics.cached_syntax_source_bytes;
    let second = SyntaxQueryKey::new("second.veac", SOURCE);
    let charged = usize::MAX - retained + 1;
    let payload = charged - second.retained_bytes() - entry_overhead().unwrap();

    cache.insert(second, payload, surface("second.veac"));

    assert_eq!(cache.statistics.syntax_bypasses, 1);
    assert_eq!(cache.statistics.syntax_evictions, 0);
    assert_eq!(cache.values.len(), 1);
    assert_eq!(cache.statistics.cached_syntax_source_bytes, retained);
}

#[test]
fn clear_releases_fifo_backing_allocation() {
    let mut cache = unbounded_cache();
    for index in 0..128 {
        let path = format!("{index}.veac");
        cache.insert(SyntaxQueryKey::new(&path, SOURCE), 0, surface(&path));
    }
    assert!(cache.insertion_order.capacity() > 0);

    cache.clear();

    assert_eq!(cache.insertion_order.capacity(), 0);
    assert!(cache.values.is_empty());
    assert_eq!(cache.statistics.cached_syntax_entries, 0);
    assert_eq!(cache.statistics.cached_syntax_source_bytes, 0);
}

fn unbounded_cache() -> SyntaxCache {
    SyntaxCache::new(CompilerDatabaseLimits {
        syntax_entries: usize::MAX,
        syntax_source_bytes: usize::MAX,
        ..Default::default()
    })
}

fn surface(path: &str) -> Arc<crate::program::model::SurfaceFile> {
    Arc::new(crate::program::parser::parse(path, SOURCE).unwrap())
}
