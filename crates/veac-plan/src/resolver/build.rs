use std::collections::BTreeSet;

use crate::{
    PlanSource, RenderPlanHeader, ResolutionDiagnostic, ResolvedRenderPlan, ResolverFingerprint,
    CURRENT_RENDER_PLAN_VERSION, RENDER_PLAN_SCHEMA_ID, RESOLVER_VERSION,
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
        let plan = ResolvedRenderPlan {
            header: self.header(policies.into_iter().collect::<Vec<_>>().join(",")),
            output: self.output(output_id),
            inputs: self.inputs.into_values().collect(),
            sequences,
            entry_sequence_id: self.config.sequence_id.clone(),
        };
        (Some(plan), self.diagnostics)
    }

    fn header(&self, policies: String) -> RenderPlanHeader {
        RenderPlanHeader {
            schema: RENDER_PLAN_SCHEMA_ID.to_owned(),
            schema_version: CURRENT_RENDER_PLAN_VERSION,
            source: PlanSource {
                project_id: self.envelope.project.id.clone(),
                revision: self.envelope.project.revision,
                timebase: self.envelope.project.timebase,
                semantic_hash: self.semantic_hash.to_owned(),
                snapshot_hash: self.snapshot_hash.to_owned(),
            },
            resolver: ResolverFingerprint {
                resolver_version: RESOLVER_VERSION.to_owned(),
                stream_selection_policy: if policies.is_empty() {
                    "none".to_owned()
                } else {
                    policies
                },
                effect_registry_version: "veac-ir-effects-v1".to_owned(),
                capability_profile: "backend-neutral-v1".to_owned(),
            },
        }
    }
}
