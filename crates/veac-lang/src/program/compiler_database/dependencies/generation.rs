use std::sync::Arc;

use super::{CompilerSourceRevision, DependencyGraph};

#[derive(Clone, Debug)]
pub(crate) struct ObservationToken(Arc<()>);

#[derive(Clone, Debug)]
pub(crate) struct RevisionGeneration(Arc<()>);

#[derive(Clone, Debug)]
pub(crate) struct InvalidationGeneration(Arc<()>);

impl ObservationToken {
    pub(super) fn new() -> Self {
        Self(Arc::new(()))
    }

    fn matches(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl InvalidationGeneration {
    pub(super) fn new() -> Self {
        Self(Arc::new(()))
    }

    fn matches(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl RevisionGeneration {
    pub(super) fn new() -> Self {
        Self(Arc::new(()))
    }

    fn matches(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl DependencyGraph {
    pub(crate) fn observation_token(&self, source_id: &str) -> Option<ObservationToken> {
        self.retained
            .current
            .get(source_id)
            .map(|state| state.observation.clone())
    }

    pub(crate) fn invalidation_generation(
        &self,
        source_id: &str,
    ) -> Option<InvalidationGeneration> {
        self.retained
            .current
            .get(source_id)
            .and_then(|state| state.invalidation.clone())
    }

    pub(crate) fn revision_generation(&self, source_id: &str) -> Option<RevisionGeneration> {
        self.retained
            .current
            .get(source_id)
            .map(|state| state.revision_generation.clone())
    }

    pub(super) fn revision_generation_matches(
        &self,
        source_id: &str,
        expected: Option<&RevisionGeneration>,
    ) -> bool {
        self.retained
            .current
            .get(source_id)
            .zip(expected)
            .is_some_and(|(current, expected)| current.revision_generation.matches(expected))
    }

    pub(super) fn observation_matches(
        &self,
        source_id: &str,
        expected: Option<&ObservationToken>,
    ) -> bool {
        self.retained
            .current
            .get(source_id)
            .zip(expected)
            .is_some_and(|(current, expected)| current.observation.matches(expected))
    }

    fn invalidation_matches(
        &self,
        source_id: &str,
        expected: Option<&InvalidationGeneration>,
    ) -> bool {
        self.retained
            .current
            .get(source_id)
            .and_then(|state| state.invalidation.as_ref())
            .zip(expected)
            .is_some_and(|(current, expected)| current.matches(expected))
    }

    pub(crate) fn consume(
        &mut self,
        source_id: &str,
        revision: CompilerSourceRevision,
        revision_generation: Option<&RevisionGeneration>,
        observation: Option<&ObservationToken>,
        generation: Option<&InvalidationGeneration>,
    ) -> bool {
        if self
            .retained
            .current
            .get(source_id)
            .map(|state| state.revision)
            != Some(revision)
            || !self.revision_generation_matches(source_id, revision_generation)
            || !self.observation_matches(source_id, observation)
            || !self.invalidation_matches(source_id, generation)
        {
            return false;
        }
        if !self.retained.pending.remove(source_id) {
            return false;
        }
        self.retained
            .current
            .get_mut(source_id)
            .expect("admitted invalidation belongs to retained source")
            .invalidation = None;
        true
    }

    pub(super) fn invalidate(&mut self, source_ids: Vec<String>) {
        for source_id in source_ids {
            if let Some(state) = self.retained.current.get_mut(&source_id) {
                state.invalidation = Some(InvalidationGeneration::new());
                self.retained.pending.insert(source_id);
            }
        }
    }
}

#[cfg(test)]
#[path = "generation_tests.rs"]
mod tests;
