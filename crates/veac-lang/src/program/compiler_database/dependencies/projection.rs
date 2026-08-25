use std::collections::{BTreeMap, BTreeSet};

use super::{CompilerDependencyRoute, CompilerDependencySnapshot, DependencyGraph};
use crate::program::CompilerDatabaseStatistics;

impl DependencyGraph {
    pub(crate) fn snapshot(&self, source_id: &str) -> Option<CompilerDependencySnapshot> {
        Some(CompilerDependencySnapshot {
            source_id: source_id.to_owned(),
            revision: self.retained.current.get(source_id)?.revision,
            direct_dependencies: values(&self.retained.dependencies, source_id),
            routes: self
                .retained
                .routes
                .get(source_id)
                .into_iter()
                .flatten()
                .map(|(requested, resolved)| CompilerDependencyRoute {
                    importer: source_id.to_owned(),
                    requested_path: requested.clone(),
                    resolved_source_id: resolved.clone(),
                })
                .collect(),
        })
    }

    pub(crate) fn pending(&self) -> Vec<String> {
        self.retained.pending.iter().cloned().collect()
    }

    pub(crate) fn contribute(&self, output: &mut CompilerDatabaseStatistics) {
        let usage = self.usage();
        output.semantic_invalidations = self.invalidations;
        output.pending_semantic_invalidations = self.retained.pending.len();
        output.dependency_route_edges = self.retained.routes.values().map(BTreeMap::len).sum();
        output.dependency_retained_entries = usage.entries;
        output.dependency_retained_bytes = usage.bytes;
        output.dependency_bypasses = self.bypasses;
        output.dependency_resets = self.resets;
        output.dependency_evictions = self.evictions;
    }
}

fn values(map: &BTreeMap<String, BTreeSet<String>>, key: &str) -> Vec<String> {
    map.get(key).into_iter().flatten().cloned().collect()
}
