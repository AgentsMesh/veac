use std::collections::BTreeMap;

use super::{
    canonical, CompilerSourceRevision, DependencyGraph, InvalidationGeneration, ObservationToken,
    RevisionGeneration,
};

pub(crate) enum RouteReplacement {
    Retained {
        reset: bool,
        revision_generation: RevisionGeneration,
        observation: ObservationToken,
        invalidation: Option<InvalidationGeneration>,
    },
    ConservativeBypass,
    RetryRequired,
}

enum WriterAdmission {
    Current,
    SupersededObservation,
    ConservativeBypass,
    RetryRequired,
}

impl DependencyGraph {
    pub(crate) fn replace_routes(
        &mut self,
        importer: &str,
        revision: CompilerSourceRevision,
        revision_generation: Option<&RevisionGeneration>,
        observation: Option<&ObservationToken>,
        routes: BTreeMap<String, String>,
    ) -> RouteReplacement {
        match self.writer_admission(
            importer,
            revision,
            revision_generation,
            observation,
            &routes,
        ) {
            WriterAdmission::Current | WriterAdmission::SupersededObservation => {}
            WriterAdmission::ConservativeBypass => {
                return RouteReplacement::ConservativeBypass;
            }
            WriterAdmission::RetryRequired => return RouteReplacement::RetryRequired,
        }
        let before = self.usage();
        self.replace_routes_unchecked(importer, routes.clone());
        if self.within_budget() {
            return self.retained_replacement(importer, self.conservative);
        }
        self.begin_reset(before.entries);
        self.observe_unchecked(importer, revision);
        self.replace_routes_unchecked(importer, routes);
        self.finish_reset();
        self.retained_replacement(importer, true)
    }

    pub(crate) fn conservative(&self) -> bool {
        self.conservative
    }

    fn writer_admission(
        &self,
        importer: &str,
        revision: CompilerSourceRevision,
        revision_generation: Option<&RevisionGeneration>,
        observation: Option<&ObservationToken>,
        routes: &BTreeMap<String, String>,
    ) -> WriterAdmission {
        let Some(state) = self.retained.current.get(importer) else {
            return if self.conservative {
                WriterAdmission::ConservativeBypass
            } else {
                WriterAdmission::RetryRequired
            };
        };
        if state.revision != revision {
            return WriterAdmission::RetryRequired;
        }
        if !self.revision_generation_matches(importer, revision_generation) {
            return WriterAdmission::RetryRequired;
        }
        if self.observation_matches(importer, observation) {
            WriterAdmission::Current
        } else if state.routes_committed && self.routes_differ(importer, routes) {
            WriterAdmission::RetryRequired
        } else {
            WriterAdmission::SupersededObservation
        }
    }

    fn routes_differ(&self, importer: &str, routes: &BTreeMap<String, String>) -> bool {
        self.retained
            .routes
            .get(importer)
            .map_or(!routes.is_empty(), |current| current != routes)
    }

    fn retained_replacement(&self, importer: &str, reset: bool) -> RouteReplacement {
        let Some(observation) = self.observation_token(importer) else {
            debug_assert!(self.conservative);
            return RouteReplacement::ConservativeBypass;
        };
        RouteReplacement::Retained {
            reset,
            revision_generation: self
                .revision_generation(importer)
                .expect("retained route writer has a revision generation"),
            observation,
            invalidation: self.invalidation_generation(importer),
        }
    }

    fn replace_routes_unchecked(&mut self, importer: &str, routes: BTreeMap<String, String>) {
        let routes = routes
            .into_iter()
            .map(|(requested, resolved)| (canonical(&requested), canonical(&resolved)))
            .collect::<BTreeMap<_, _>>();
        let changed = self
            .retained
            .routes
            .get(importer)
            .is_some_and(|previous| previous != &routes);
        let dependencies = routes.values().cloned().collect();
        if routes.is_empty() {
            self.retained.routes.remove(importer);
        } else {
            self.retained.routes.insert(canonical(importer), routes);
        }
        self.replace_dependencies(importer, dependencies);
        self.retained
            .current
            .get_mut(importer)
            .expect("admitted route writer is retained")
            .routes_committed = true;
        if changed {
            self.invalidate_source(importer);
        }
    }
}
