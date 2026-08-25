use std::sync::Arc;

use super::bounded_cache::BoundedCache;
use super::interface_key::InterfaceQueryKey;
use super::statistics::CompilerDatabaseStatistics;
use crate::program::ModuleInterface;

#[derive(Debug)]
pub(super) struct InterfaceCache {
    values: BoundedCache<InterfaceQueryKey, ModuleInterface>,
    stats: InterfaceStats,
}

#[derive(Debug, Default)]
struct InterfaceStats {
    hits: u64,
    misses: u64,
    insertions: u64,
    evictions: u64,
    bypasses: u64,
}

impl InterfaceCache {
    pub(super) fn new(entries: usize, retained_bytes: usize) -> Self {
        Self {
            values: BoundedCache::new(entries, retained_bytes),
            stats: InterfaceStats::default(),
        }
    }

    pub(super) fn get(&mut self, key: &InterfaceQueryKey) -> Option<Arc<ModuleInterface>> {
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
        key: InterfaceQueryKey,
        retained_bytes: Option<usize>,
        value: Arc<ModuleInterface>,
    ) -> Arc<ModuleInterface> {
        let Some(retained_bytes) =
            retained_bytes.and_then(|bytes| bytes.checked_add(key.retained_bytes()))
        else {
            self.stats.bypasses = self.stats.bypasses.saturating_add(1);
            return value;
        };
        let outcome = self.values.insert(key, retained_bytes, value);
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

    pub(super) fn contribute(&self, output: &mut CompilerDatabaseStatistics) {
        output.interface_hits = self.stats.hits;
        output.interface_misses = self.stats.misses;
        output.interface_insertions = self.stats.insertions;
        output.interface_evictions = self.stats.evictions;
        output.interface_bypasses = self.stats.bypasses;
        output.cached_interface_entries = self.values.len();
        output.cached_interface_retained_bytes = self.values.retained_bytes();
    }
}
