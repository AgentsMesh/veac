use std::collections::BTreeSet;

use crate::{
    PlanSource, RenderPlanHeader, ResolutionDiagnostic, ResolutionErrorKind, ResolvedRenderPlan,
    ResolverFingerprint, CAPABILITY_PROFILE, CURRENT_RENDER_PLAN_VERSION, EFFECT_REGISTRY_VERSION,
    RENDER_PLAN_SCHEMA_ID, RESOLVER_VERSION,
};

use super::{error::normalize_diagnostics, PlanResolver};

impl PlanResolver<'_> {
    pub(super) fn build(mut self) -> (Option<ResolvedRenderPlan>, Vec<ResolutionDiagnostic>) {
        self.resolve_sequence(&self.config.sequence_id);
        let Some(output_id) = self.output_id() else {
            normalize_diagnostics(&mut self.diagnostics);
            return (None, self.diagnostics);
        };
        if !self.diagnostics.is_empty() {
            normalize_diagnostics(&mut self.diagnostics);
            return (None, self.diagnostics);
        }
        let mut sequences = Vec::with_capacity(self.sequence_order.len());
        for id in &self.sequence_order {
            if let Some(sequence) = self.sequences.remove(id) {
                sequences.push(sequence);
            }
        }
        let policies: BTreeSet<_> = self
            .inputs
            .values()
            .filter_map(|input| input.probe.as_ref())
            .map(|probe| probe.selection_policy.as_str())
            .collect();
        let temporal = match crate::temporal::select(self.envelope, &sequences) {
            Ok(value) => value,
            Err(issues) => {
                for issue in issues {
                    self.push(ResolutionDiagnostic::new(
                        ResolutionErrorKind::InternalInvariant,
                        issue.code,
                        None,
                        issue.pointer,
                        issue.message,
                    ));
                }
                normalize_diagnostics(&mut self.diagnostics);
                return (None, self.diagnostics);
            }
        };
        let policies = policies.into_iter().collect::<Vec<_>>().join(",");
        let source = self.source();
        let resolver = self.resolver(policies);
        let cache = match crate::identity::cache(&source, &resolver, &temporal) {
            Ok(value) => value,
            Err(error) => {
                self.push_internal(
                    "PLAN_CACHE_IDENTITY",
                    self.envelope.project.id.to_string(),
                    error.to_string(),
                );
                return (None, self.diagnostics);
            }
        };
        let plan = ResolvedRenderPlan {
            header: RenderPlanHeader {
                schema: RENDER_PLAN_SCHEMA_ID.to_owned(),
                schema_version: CURRENT_RENDER_PLAN_VERSION,
                source,
                resolver,
                cache,
            },
            output: self.output(output_id),
            inputs: self.inputs.into_values().collect(),
            temporal,
            sequences,
            entry_sequence_id: self.config.sequence_id.clone(),
        };
        if let Err(errors) = crate::validate_render_plan(&plan) {
            let diagnostics = errors
                .into_diagnostics()
                .into_iter()
                .map(|value| {
                    ResolutionDiagnostic::new(
                        ResolutionErrorKind::RenderPlanContract,
                        &value.code,
                        None,
                        value.pointer,
                        value.message,
                    )
                })
                .collect();
            return (None, diagnostics);
        }
        (Some(plan), self.diagnostics)
    }

    fn source(&self) -> PlanSource {
        PlanSource {
            project_id: self.envelope.project.id.clone(),
            revision: self.envelope.project.revision,
            timebase: self.envelope.project.timebase,
            semantic_hash: self.semantic_hash.to_owned(),
            snapshot_hash: self.snapshot_hash.to_owned(),
            executable: self.envelope.executable.clone(),
        }
    }

    fn resolver(&self, policies: String) -> ResolverFingerprint {
        ResolverFingerprint {
            resolver_version: RESOLVER_VERSION.to_owned(),
            stream_selection_policy: if policies.is_empty() {
                "none".to_owned()
            } else {
                policies
            },
            effect_registry_version: EFFECT_REGISTRY_VERSION.to_owned(),
            capability_profile: CAPABILITY_PROFILE.to_owned(),
        }
    }
}
