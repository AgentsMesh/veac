use std::collections::BTreeSet;
use std::mem::size_of;

use super::{DependencyGraph, RetainedGraph};

#[derive(Clone, Copy)]
pub(super) struct Usage {
    pub(super) entries: usize,
    pub(super) bytes: usize,
    overflowed: bool,
}

impl DependencyGraph {
    pub(super) fn usage(&self) -> Usage {
        let mut usage = Usage {
            entries: 0,
            bytes: 0,
            overflowed: false,
        };
        for source in self.retained.current.keys() {
            usage.map_entry::<String, super::SourceState>(source.len());
        }
        for (source, dependencies) in &self.retained.dependencies {
            usage.map_entry::<String, BTreeSet<String>>(source.len());
            usage.string_set(dependencies);
        }
        for (source, importers) in &self.retained.reverse {
            usage.map_entry::<String, BTreeSet<String>>(source.len());
            usage.string_set(importers);
        }
        for (importer, routes) in &self.retained.routes {
            usage.map_entry::<String, std::collections::BTreeMap<String, String>>(importer.len());
            for (requested, resolved) in routes {
                usage.map_entry::<String, String>(requested.len());
                usage.add_bytes(resolved.len());
            }
        }
        usage.string_set(&self.retained.pending);
        if usage.entries > 0 {
            usage.add_bytes(size_of::<RetainedGraph>());
        }
        usage
    }

    pub(super) fn within_budget(&self) -> bool {
        self.fits(self.usage())
    }

    fn fits(&self, usage: Usage) -> bool {
        self.entry_limit > 0
            && self.byte_limit > 0
            && !usage.overflowed
            && usage.entries <= self.entry_limit
            && usage.bytes <= self.byte_limit
    }

    pub(super) fn begin_reset(&mut self, evictions: usize) {
        self.resets = self.resets.saturating_add(1);
        self.evictions = self
            .evictions
            .saturating_add(u64::try_from(evictions).unwrap_or(u64::MAX));
        self.retained = RetainedGraph::default();
        self.conservative = true;
    }

    pub(super) fn finish_reset(&mut self) {
        if !self.within_budget() {
            self.retained = RetainedGraph::default();
            self.bypasses = self.bypasses.saturating_add(1);
        }
    }

    pub(super) fn replace_dependencies(&mut self, source_id: &str, dependencies: BTreeSet<String>) {
        if let Some(previous) = self.retained.dependencies.remove(source_id) {
            for dependency in previous {
                let empty = self
                    .retained
                    .reverse
                    .get_mut(&dependency)
                    .is_some_and(|values| {
                        values.remove(source_id);
                        values.is_empty()
                    });
                if empty {
                    self.retained.reverse.remove(&dependency);
                }
            }
        }
        if !dependencies.is_empty() {
            self.retained
                .dependencies
                .insert(super::canonical(source_id), dependencies.clone());
        }
        for dependency in dependencies {
            self.retained
                .reverse
                .entry(dependency)
                .or_default()
                .insert(super::canonical(source_id));
        }
    }

    pub(super) fn transitive_dependents(&self, source_id: &str) -> Vec<String> {
        let mut work = vec![source_id.to_owned()];
        let mut output = BTreeSet::new();
        while let Some(current) = work.pop() {
            for dependent in self.retained.reverse.get(&current).into_iter().flatten() {
                if output.insert(dependent.clone()) {
                    work.push(dependent.clone());
                }
            }
        }
        output.into_iter().collect()
    }

    pub(super) fn invalidate_source(&mut self, source_id: &str) {
        let mut affected = self.transitive_dependents(source_id);
        affected.push(source_id.to_owned());
        self.invalidations = self
            .invalidations
            .saturating_add(u64::try_from(affected.len()).unwrap_or(u64::MAX));
        self.invalidate(affected);
    }
}

impl Usage {
    fn map_entry<K, V>(&mut self, dynamic: usize) {
        self.entries = match self.entries.checked_add(1) {
            Some(value) => value,
            None => {
                self.overflowed = true;
                self.entries
            }
        };
        self.add_bytes(size_of::<K>());
        self.add_bytes(size_of::<V>());
        self.add_bytes(dynamic);
        self.add_bytes(512);
    }

    fn string_set(&mut self, values: &BTreeSet<String>) {
        for value in values {
            self.map_entry::<String, ()>(value.len());
        }
    }

    fn add_bytes(&mut self, added: usize) {
        self.bytes = match self.bytes.checked_add(added) {
            Some(value) => value,
            None => {
                self.overflowed = true;
                self.bytes
            }
        };
    }
}

#[cfg(test)]
#[path = "budget/tests.rs"]
mod tests;
