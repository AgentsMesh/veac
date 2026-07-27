mod apply;
mod apply_ranges;
mod apply_target;
mod bounds;
mod clip;
mod color;
mod components;
mod defaults;
mod effects;
mod error;
mod font;
mod material;
mod multicam;
mod output;
mod relation_properties;
mod resource;
mod selection;
mod source_time;
mod streams;
mod time;
mod timeline;
mod transition;

#[cfg(test)]
mod unit_tests;

use std::collections::{BTreeMap, BTreeSet};

use veac_ir::RelationGraph;

use veac_ir::{ProjectEnvelope, RenderConfig, RenderConfigId, SequenceId};

use crate::{
    PlanOutputId, PlanSource, RenderPlanHeader, ResolutionDiagnostic, ResolutionErrorKind,
    ResolutionErrors, ResolvedInput, ResolvedRenderPlan, ResolvedSequence, ResolverFingerprint,
    CURRENT_RENDER_PLAN_VERSION, RENDER_PLAN_SCHEMA_ID, RESOLVER_VERSION,
};

use error::{canonical_errors, internal_hash_error, normalize_diagnostics};
use selection::select_configs;

/// Resolve all configs, or one selected config, without performing environment I/O.
///
/// Returned plans are sorted by render-config ID. Disabled clips and clips on non-solo tracks
/// under an active solo are omitted before material requirements are evaluated.
pub fn resolve(
    envelope: &ProjectEnvelope,
    render_config_id: Option<&RenderConfigId>,
) -> Result<Vec<ResolvedRenderPlan>, ResolutionErrors> {
    if let Err(errors) = veac_ir::validate(envelope) {
        return Err(canonical_errors(errors));
    }
    let configs = select_configs(envelope, render_config_id)?;
    let semantic = veac_ir::semantic_hash(envelope).map_err(internal_hash_error)?;
    let snapshot = veac_ir::snapshot_hash(envelope).map_err(internal_hash_error)?;
    let mut plans = Vec::with_capacity(configs.len());
    let mut diagnostics = Vec::new();
    for config in configs {
        let resolver = PlanResolver::new(envelope, config, &semantic, &snapshot);
        let (plan, mut errors) = resolver.build();
        diagnostics.append(&mut errors);
        plans.extend(plan);
    }
    normalize_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        Ok(plans)
    } else {
        Err(ResolutionErrors::new(diagnostics))
    }
}

/// Resolve exactly one render config.
pub fn resolve_one(
    envelope: &ProjectEnvelope,
    render_config_id: &RenderConfigId,
) -> Result<ResolvedRenderPlan, ResolutionErrors> {
    let mut plans = resolve(envelope, Some(render_config_id))?;
    plans.pop().ok_or_else(|| {
        ResolutionErrors::new(vec![ResolutionDiagnostic::new(
            ResolutionErrorKind::InternalInvariant,
            "EMPTY_PLAN_RESULT",
            Some(render_config_id.to_string()),
            "/project/render_configs",
            "selected render config did not produce a plan",
        )])
    })
}

pub(super) struct PlanResolver<'a> {
    envelope: &'a ProjectEnvelope,
    relations: RelationGraph<'a>,
    config: &'a RenderConfig,
    semantic_hash: &'a str,
    snapshot_hash: &'a str,
    inputs: BTreeMap<crate::PlanInputId, ResolvedInput>,
    sequences: BTreeMap<SequenceId, ResolvedSequence>,
    sequence_order: Vec<SequenceId>,
    visiting: BTreeSet<SequenceId>,
    diagnostics: Vec<ResolutionDiagnostic>,
}

impl<'a> PlanResolver<'a> {
    fn new(
        envelope: &'a ProjectEnvelope,
        config: &'a RenderConfig,
        semantic_hash: &'a str,
        snapshot_hash: &'a str,
    ) -> Self {
        Self {
            envelope,
            relations: RelationGraph::project(&envelope.project),
            config,
            semantic_hash,
            snapshot_hash,
            inputs: BTreeMap::new(),
            sequences: BTreeMap::new(),
            sequence_order: Vec::new(),
            visiting: BTreeSet::new(),
            diagnostics: Vec::new(),
        }
    }

    fn build(mut self) -> (Option<ResolvedRenderPlan>, Vec<ResolutionDiagnostic>) {
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

    fn output_id(&mut self) -> Option<PlanOutputId> {
        let suffix = self.config.id.as_str().strip_prefix("out_")?;
        match PlanOutputId::new(format!("pout_{suffix}")) {
            Ok(id) => Some(id),
            Err(error) => {
                self.push_internal(
                    "PLAN_OUTPUT_ID",
                    self.config.id.to_string(),
                    error.to_string(),
                );
                None
            }
        }
    }
}
