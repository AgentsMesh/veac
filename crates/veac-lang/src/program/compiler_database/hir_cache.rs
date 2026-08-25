use std::sync::Arc;

use super::bounded_cache::BoundedCache;
use super::statistics::CompilerDatabaseStatistics;
use crate::program::expression::{FunctionQueryKey, TypedFunctionBatch};

#[derive(Debug)]
pub(super) struct HirCache {
    values: BoundedCache<FunctionQueryKey, TypedFunctionBatch>,
    stats: SemanticStats,
}

#[derive(Debug, Default)]
struct SemanticStats {
    hits: u64,
    misses: u64,
    insertions: u64,
    evictions: u64,
    bypasses: u64,
}

impl HirCache {
    pub(super) fn new(entries: usize, retained_bytes: usize) -> Self {
        Self {
            values: BoundedCache::new(entries, retained_bytes),
            stats: SemanticStats::default(),
        }
    }

    pub(super) fn get(&mut self, key: &FunctionQueryKey) -> Option<Arc<TypedFunctionBatch>> {
        match self.values.get(key) {
            Some(value) => {
                self.stats.hits = self.stats.hits.saturating_add(1);
                Some(value)
            }
            None => {
                self.stats.misses = self.stats.misses.saturating_add(1);
                None
            }
        }
    }

    pub(super) fn insert(
        &mut self,
        key: FunctionQueryKey,
        bytes: Option<usize>,
        value: Arc<TypedFunctionBatch>,
    ) -> Arc<TypedFunctionBatch> {
        let Some(bytes) = bytes else {
            self.stats.bypasses = self.stats.bypasses.saturating_add(1);
            return value;
        };
        let outcome = self.values.insert(key, bytes, value);
        self.stats.insertions = self
            .stats
            .insertions
            .saturating_add(u64::from(outcome.inserted));
        self.stats.bypasses = self
            .stats
            .bypasses
            .saturating_add(u64::from(outcome.bypassed));
        self.stats.evictions = self.stats.evictions.saturating_add(outcome.evictions);
        outcome.value
    }

    pub(super) fn clear(&mut self) {
        self.values.clear();
    }

    pub(super) fn bypass(&mut self) {
        self.stats.bypasses = self.stats.bypasses.saturating_add(1);
    }

    pub(super) fn contribute(&self, stats: &mut CompilerDatabaseStatistics) {
        stats.hir_hits = self.stats.hits;
        stats.hir_misses = self.stats.misses;
        stats.hir_insertions = self.stats.insertions;
        stats.hir_evictions = self.stats.evictions;
        stats.hir_bypasses = self.stats.bypasses;
        stats.cached_hir_entries = self.values.len();
        stats.cached_hir_retained_bytes = self.values.retained_bytes();
    }
}
