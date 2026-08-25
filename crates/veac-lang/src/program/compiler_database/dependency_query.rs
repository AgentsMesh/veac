use std::collections::BTreeMap;

use super::dependencies::{
    InvalidationGeneration, ObservationToken, RevisionGeneration, RouteReplacement,
};
use super::lifecycle::QueryEpoch;
use super::{CompilerDatabase, CompilerDependencySnapshot, CompilerSourceRevision};

#[derive(Clone, Debug)]
pub(crate) struct DependencyRouteAdmission {
    importer: String,
    revision: CompilerSourceRevision,
    epoch: QueryEpoch,
    revision_generation: Option<RevisionGeneration>,
    observation: Option<ObservationToken>,
    invalidation: Option<InvalidationGeneration>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DependencyRouteCommit {
    Committed,
    ConservativeBypass,
    RetryRequired,
}

pub(crate) const DEPENDENCY_ROUTE_ATTEMPTS: usize = 4;

impl DependencyRouteAdmission {
    pub(super) fn epoch(&self) -> &QueryEpoch {
        &self.epoch
    }
}

impl CompilerDatabase {
    pub fn dependency_snapshot(&self, source_id: &str) -> Option<CompilerDependencySnapshot> {
        let _lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        self.dependencies
            .lock()
            .expect("dependency graph lock poisoned")
            .snapshot(source_id)
    }

    pub fn pending_invalidations(&self) -> Vec<String> {
        let _lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        self.dependencies
            .lock()
            .expect("dependency graph lock poisoned")
            .pending()
    }

    pub(crate) fn consume_invalidation(&self, admission: &DependencyRouteAdmission) -> bool {
        let _lifecycle = self
            .lifecycle
            .read()
            .expect("compiler database lifecycle lock poisoned");
        if !_lifecycle.admits(&admission.epoch) {
            return false;
        }
        self.dependencies
            .lock()
            .expect("dependency graph lock poisoned")
            .consume(
                &admission.importer,
                admission.revision,
                admission.revision_generation.as_ref(),
                admission.observation.as_ref(),
                admission.invalidation.as_ref(),
            )
    }

    pub(crate) fn commit_routes(
        &self,
        admission: &mut DependencyRouteAdmission,
        routes: BTreeMap<String, String>,
    ) -> DependencyRouteCommit {
        let mut lifecycle = self
            .lifecycle
            .write()
            .expect("compiler database lifecycle lock poisoned");
        if !lifecycle.same_release(&admission.epoch) {
            return DependencyRouteCommit::RetryRequired;
        }
        if !lifecycle.admits(&admission.epoch) {
            let conservative = self
                .dependencies
                .lock()
                .expect("dependency graph lock poisoned")
                .conservative();
            return if conservative {
                DependencyRouteCommit::ConservativeBypass
            } else {
                DependencyRouteCommit::RetryRequired
            };
        }
        let replacement = self
            .dependencies
            .lock()
            .expect("dependency graph lock poisoned")
            .replace_routes(
                &admission.importer,
                admission.revision,
                admission.revision_generation.as_ref(),
                admission.observation.as_ref(),
                routes,
            );
        match replacement {
            RouteReplacement::Retained {
                reset,
                revision_generation,
                observation,
                invalidation,
            } => {
                self.reset_semantic_caches(&mut lifecycle, reset);
                admission.epoch = lifecycle.epoch();
                admission.revision_generation = Some(revision_generation);
                admission.observation = Some(observation);
                admission.invalidation = invalidation;
                DependencyRouteCommit::Committed
            }
            RouteReplacement::ConservativeBypass => {
                self.reset_semantic_caches(&mut lifecycle, true);
                admission.epoch = lifecycle.epoch();
                admission.revision_generation = None;
                admission.observation = None;
                admission.invalidation = None;
                DependencyRouteCommit::ConservativeBypass
            }
            RouteReplacement::RetryRequired => DependencyRouteCommit::RetryRequired,
        }
    }

    pub(super) fn observe_dependency(
        &self,
        source_id: &str,
        revision: CompilerSourceRevision,
    ) -> DependencyRouteAdmission {
        let mut lifecycle = self
            .lifecycle
            .write()
            .expect("compiler database lifecycle lock poisoned");
        let (reset, observation, revision_generation, invalidation) = {
            let mut dependencies = self
                .dependencies
                .lock()
                .expect("dependency graph lock poisoned");
            let reset = dependencies.observe(source_id, revision);
            (
                reset,
                dependencies.observation_token(source_id),
                dependencies.revision_generation(source_id),
                dependencies.invalidation_generation(source_id),
            )
        };
        self.reset_semantic_caches(&mut lifecycle, reset);
        DependencyRouteAdmission {
            importer: source_id.to_owned(),
            revision,
            epoch: lifecycle.epoch(),
            revision_generation,
            observation,
            invalidation,
        }
    }

    pub(crate) fn dependency_route_admission(
        &self,
        source_id: &str,
        source: &str,
    ) -> DependencyRouteAdmission {
        self.observe_dependency(source_id, CompilerSourceRevision::new(source_id, source))
    }

    fn reset_semantic_caches(&self, lifecycle: &mut super::lifecycle::Lifecycle, reset: bool) {
        if !reset {
            return;
        }
        lifecycle.advance();
        self.interface
            .lock()
            .expect("interface cache lock poisoned")
            .clear();
        self.hir.lock().expect("HIR cache lock poisoned").clear();
        self.core.lock().expect("core cache lock poisoned").clear();
    }
}
