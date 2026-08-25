mod budget;
mod generation;
mod projection;
mod routes;

use std::collections::{BTreeMap, BTreeSet};

use super::revision::CompilerSourceRevision;
pub(crate) use generation::{InvalidationGeneration, ObservationToken, RevisionGeneration};
pub(crate) use routes::RouteReplacement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerDependencySnapshot {
    pub source_id: String,
    pub revision: CompilerSourceRevision,
    pub direct_dependencies: Vec<String>,
    pub routes: Vec<CompilerDependencyRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompilerDependencyRoute {
    pub importer: String,
    pub requested_path: String,
    pub resolved_source_id: String,
}

#[derive(Debug, Default)]
struct RetainedGraph {
    current: BTreeMap<String, SourceState>,
    dependencies: BTreeMap<String, BTreeSet<String>>,
    reverse: BTreeMap<String, BTreeSet<String>>,
    routes: BTreeMap<String, BTreeMap<String, String>>,
    pending: BTreeSet<String>,
}

#[derive(Debug)]
struct SourceState {
    revision: CompilerSourceRevision,
    revision_generation: RevisionGeneration,
    observation: ObservationToken,
    invalidation: Option<InvalidationGeneration>,
    routes_committed: bool,
}

#[derive(Debug)]
pub(super) struct DependencyGraph {
    retained: RetainedGraph,
    entry_limit: usize,
    byte_limit: usize,
    invalidations: u64,
    bypasses: u64,
    resets: u64,
    evictions: u64,
    conservative: bool,
}

impl DependencyGraph {
    pub(super) fn new(entry_limit: usize, byte_limit: usize) -> Self {
        Self {
            retained: RetainedGraph::default(),
            entry_limit,
            byte_limit,
            invalidations: 0,
            bypasses: 0,
            resets: 0,
            evictions: 0,
            conservative: false,
        }
    }

    pub(super) fn observe(&mut self, source_id: &str, revision: CompilerSourceRevision) -> bool {
        let before = self.usage();
        self.observe_unchecked(source_id, revision);
        if self.within_budget() {
            return self.conservative;
        }
        self.begin_reset(before.entries);
        self.observe_unchecked(source_id, revision);
        self.finish_reset();
        true
    }

    pub(super) fn clear(&mut self) {
        self.retained = RetainedGraph::default();
        self.conservative = false;
    }

    fn observe_unchecked(&mut self, source_id: &str, revision: CompilerSourceRevision) {
        let previous_revision = self
            .retained
            .current
            .get(source_id)
            .map(|state| state.revision);
        let invalidation = self
            .retained
            .current
            .get(source_id)
            .and_then(|state| state.invalidation.clone());
        let revision_generation = self
            .retained
            .current
            .get(source_id)
            .filter(|state| state.revision == revision)
            .map_or_else(RevisionGeneration::new, |state| {
                state.revision_generation.clone()
            });
        let routes_committed = self
            .retained
            .current
            .get(source_id)
            .is_some_and(|state| state.revision == revision && state.routes_committed);
        self.retained.current.insert(
            canonical(source_id),
            SourceState {
                revision,
                revision_generation,
                observation: ObservationToken::new(),
                invalidation,
                routes_committed,
            },
        );
        if previous_revision.is_none() || previous_revision == Some(revision) {
            return;
        }
        let invalidated = self.transitive_dependents(source_id);
        self.invalidations = self
            .invalidations
            .saturating_add(u64::try_from(invalidated.len()).unwrap_or(u64::MAX));
        self.invalidate(invalidated);
    }
}

fn canonical(value: &str) -> String {
    value.to_owned().into_boxed_str().into_string()
}
