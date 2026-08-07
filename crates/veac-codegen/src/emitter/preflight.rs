mod animation;
mod apply;
mod apply_target;
mod audio;
mod budget;
mod color;
mod composition;
mod effects;
mod generated;
mod input_streams;
mod inputs;
mod multicam;
mod output;
mod plan_contract;
mod references;
mod resource;
mod source_time;
mod structure;
mod text;
mod transitions;
mod visual;

use std::collections::BTreeSet;

use veac_plan::canonical::{RationalTime, TimeRange};
use veac_plan::{PlanInputId, ResolvedRenderPlan, ResolvedSequence};

use veac_artifact::ExecutionBindings;

use super::error::{diagnostic, CodegenErrorKind};
use super::CodegenErrors;

pub(super) fn validate(plan: &ResolvedRenderPlan) -> Result<(), CodegenErrors> {
    let mut check = Check::default();
    plan_contract::validate(&mut check, plan);
    check.header(plan);
    budget::validate_structural(&mut check, plan);
    inputs::validate(&mut check, plan);
    structure::validate(&mut check, plan);
    text::validate(&mut check, plan);
    output::validate(&mut check, plan);
    references::references(&mut check, plan);
    input_streams::validate(&mut check, plan);
    multicam::validate(&mut check, plan);
    source_time::validate(&mut check, plan);
    check.sequences(plan);
    apply::validate(plan, &mut check);
    effects::validate(&mut check, plan);
    animation::validate(&mut check, plan);
    references::cycles(&mut check, plan);
    budget::validate_visual(&mut check, plan);
    match CodegenErrors::new(check.diagnostics) {
        Some(errors) => Err(errors),
        None => Ok(()),
    }
}

pub(super) fn validate_resources(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    used: &BTreeSet<PlanInputId>,
) -> Result<(), CodegenErrors> {
    resource::validate(plan, bindings, used)
}

#[derive(Default)]
pub(super) struct Check {
    diagnostics: Vec<super::CodegenDiagnostic>,
}

impl Check {
    fn header(&mut self, plan: &ResolvedRenderPlan) {
        if plan.header.schema != veac_plan::RENDER_PLAN_SCHEMA_ID
            || plan.header.schema_version != veac_plan::CURRENT_RENDER_PLAN_VERSION
        {
            self.push(
                "PLAN_SCHEMA_UNSUPPORTED",
                None,
                "render plan schema or version is unsupported",
            );
        }
        let source = &plan.header.source;
        let resolver = &plan.header.resolver;
        if veac_plan::canonical::ProjectId::new(source.project_id.as_str()).is_err()
            || source.revision > veac_plan::canonical::MAX_SAFE_INTEGER
            || source.timebase == 0
            || !digest_valid(&source.semantic_hash)
            || !digest_valid(&source.snapshot_hash)
            || resolver.resolver_version.trim().is_empty()
            || resolver.stream_selection_policy.trim().is_empty()
            || resolver.effect_registry_version != veac_plan::EFFECT_REGISTRY_VERSION
            || resolver.capability_profile != veac_plan::CAPABILITY_PROFILE
        {
            self.push(
                "PLAN_HEADER_INVALID",
                Some(source.project_id.to_string()),
                "render plan source identity or resolver fingerprint is invalid",
            );
        }
    }

    fn sequences(&mut self, plan: &ResolvedRenderPlan) {
        let mut ids = BTreeSet::new();
        for sequence in &plan.sequences {
            if !ids.insert(&sequence.id) {
                self.push(
                    "PLAN_SEQUENCE_DUPLICATE",
                    Some(sequence.id.to_string()),
                    "sequence IDs must be unique",
                );
            }
            if !valid_time(sequence.duration) || sequence.duration.value <= 0 {
                self.push(
                    "PLAN_DURATION_INVALID",
                    Some(sequence.id.to_string()),
                    "sequence duration must be positive and representable",
                );
            }
            self.ranges(sequence);
            audio::validate(self, sequence, plan.header.source.timebase);
            color::validate(self, plan, sequence);
            composition::validate(self, sequence);
            generated::validate(self, sequence);
            transitions::validate(self, sequence, plan.header.source.timebase);
            visual::validate(self, sequence);
        }
    }

    fn ranges(&mut self, sequence: &ResolvedSequence) {
        for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
            if !valid_range(clip.record_range) {
                self.push(
                    "PLAN_RANGE_INVALID",
                    Some(clip.id.to_string()),
                    "clip record range is invalid",
                );
            }
        }
    }

    pub(super) fn push(
        &mut self,
        code: &'static str,
        object_id: Option<String>,
        message: &'static str,
    ) {
        self.diagnostics.push(diagnostic(
            CodegenErrorKind::InvalidPlan,
            code,
            object_id,
            message,
        ));
    }

    pub(super) fn temporal_backend(&mut self, error: super::temporal::TemporalBackendError) {
        self.diagnostics.push(diagnostic(
            CodegenErrorKind::UnsupportedTemporal,
            error.code,
            Some(error.binding_id.to_string()),
            error.message,
        ));
    }

    pub(super) fn plan_contract(&mut self, message: String) {
        self.diagnostics.push(diagnostic(
            CodegenErrorKind::InvalidPlan,
            "PLAN_CONTRACT_INVALID",
            None,
            message,
        ));
    }
}

fn valid_time(value: RationalTime) -> bool {
    value.is_valid()
}

fn valid_range(range: TimeRange) -> bool {
    valid_time(range.start)
        && valid_time(range.duration)
        && range.duration.value > 0
        && range.start.timescale == range.duration.timescale
        && range.end().is_ok()
}

fn digest_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
