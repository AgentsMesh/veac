use std::collections::{BTreeMap, VecDeque};
use std::mem::size_of;
use std::sync::Arc;

use super::key::SyntaxQueryKey;
use super::limits::CompilerDatabaseLimits;
use super::statistics::CompilerDatabaseStatistics;
use crate::program::model::SurfaceFile;

#[derive(Debug)]
pub(super) struct SyntaxCache {
    limits: CompilerDatabaseLimits,
    values: BTreeMap<SyntaxQueryKey, Entry>,
    insertion_order: VecDeque<SyntaxQueryKey>,
    statistics: CompilerDatabaseStatistics,
}

#[derive(Debug)]
struct Entry {
    source_bytes: usize,
    value: Arc<SurfaceFile>,
}

impl SyntaxCache {
    pub(super) fn new(limits: CompilerDatabaseLimits) -> Self {
        Self {
            limits,
            values: BTreeMap::new(),
            insertion_order: VecDeque::new(),
            statistics: CompilerDatabaseStatistics::default(),
        }
    }

    pub(super) fn peek(&self, key: &SyntaxQueryKey) -> Option<Arc<SurfaceFile>> {
        self.values.get(key).map(|entry| Arc::clone(&entry.value))
    }

    pub(super) fn get(&mut self, key: &SyntaxQueryKey) -> Option<Arc<SurfaceFile>> {
        match self.values.get(key) {
            Some(entry) => {
                self.statistics.syntax_hits = self.statistics.syntax_hits.saturating_add(1);
                Some(Arc::clone(&entry.value))
            }
            None => {
                self.statistics.syntax_misses = self.statistics.syntax_misses.saturating_add(1);
                None
            }
        }
    }

    pub(super) fn insert(
        &mut self,
        key: SyntaxQueryKey,
        source_bytes: usize,
        value: Arc<SurfaceFile>,
    ) -> Arc<SurfaceFile> {
        if let Some(existing) = self.values.get(&key) {
            return Arc::clone(&existing.value);
        }
        let source_bytes = entry_overhead().and_then(|overhead| {
            source_bytes
                .checked_add(key.retained_bytes())?
                .checked_add(overhead)
        });
        let Some(source_bytes) = source_bytes else {
            return self.bypass(value);
        };
        if !self.admits(source_bytes)
            || self
                .statistics
                .cached_syntax_source_bytes
                .checked_add(source_bytes)
                .is_none()
        {
            return self.bypass(value);
        }
        self.make_room(source_bytes);
        self.insertion_order.push_back(key.clone());
        self.values.insert(
            key,
            Entry {
                source_bytes,
                value: Arc::clone(&value),
            },
        );
        self.statistics.syntax_insertions = self.statistics.syntax_insertions.saturating_add(1);
        self.refresh_size();
        value
    }

    pub(super) fn stats(&self) -> CompilerDatabaseStatistics {
        self.statistics
    }

    pub(super) fn bypass(&mut self, value: Arc<SurfaceFile>) -> Arc<SurfaceFile> {
        self.statistics.syntax_bypasses = self.statistics.syntax_bypasses.saturating_add(1);
        value
    }

    pub(super) fn clear(&mut self) {
        self.values = BTreeMap::new();
        self.insertion_order = VecDeque::new();
        self.refresh_size();
    }

    fn admits(&self, source_bytes: usize) -> bool {
        self.limits.syntax_entries > 0 && source_bytes <= self.limits.syntax_source_bytes
    }

    fn make_room(&mut self, added: usize) {
        while self.values.len() >= self.limits.syntax_entries
            || exceeds(
                self.statistics.cached_syntax_source_bytes,
                added,
                self.limits.syntax_source_bytes,
            )
        {
            let Some(oldest) = self.insertion_order.pop_front() else {
                break;
            };
            if self.values.remove(&oldest).is_some() {
                self.statistics.syntax_evictions =
                    self.statistics.syntax_evictions.saturating_add(1);
                self.refresh_size();
            }
        }
    }

    fn refresh_size(&mut self) {
        self.statistics.cached_syntax_entries = self.values.len();
        self.statistics.cached_syntax_source_bytes =
            self.values.values().map(|entry| entry.source_bytes).sum();
    }
}

fn entry_overhead() -> Option<usize> {
    // BTreeMap/VecDeque do not expose allocator capacities. Charge a stable
    // upper bound for their nodes so byte budgets remain conservative.
    size_of::<SyntaxQueryKey>()
        .checked_mul(2)?
        .checked_add(size_of::<Entry>())?
        .checked_add(512)
}

fn exceeds(current: usize, added: usize, limit: usize) -> bool {
    current.checked_add(added).is_none_or(|total| total > limit)
}

#[cfg(test)]
#[path = "cache/tests.rs"]
mod tests;
