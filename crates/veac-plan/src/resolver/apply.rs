use veac_ir::{Apply, ApplyOperation, ApplyStage, TimeRange};

use crate::{
    ResolvedApply, ResolvedApplyOperation, ResolvedApplyStage, ResolvedApplyTarget, ResolvedTrack,
};

use super::{apply_ranges, apply_target, PlanResolver};

impl PlanResolver<'_> {
    pub(super) fn resolve_applies(
        &mut self,
        sequence_id: &veac_ir::SequenceId,
        authored: &[Apply],
        tracks: &[ResolvedTrack],
    ) -> Vec<ResolvedApply> {
        authored
            .iter()
            .enumerate()
            .filter(|(_, apply)| apply.enabled)
            .filter_map(|(index, apply)| {
                let source_order = self.source_order(index, apply.id.to_string())?;
                let target = apply_target::resolve(&apply.target, apply.record_range, tracks)?;
                let stages = self.resolve_apply_stages(apply, &target);
                (!stages.is_empty()).then(|| ResolvedApply {
                    id: apply.id.clone(),
                    source_order,
                    record_range: apply.record_range,
                    target,
                    stages,
                    mix: apply.mix.clone(),
                    matte: self.resolved_apply_matte(sequence_id, &apply.id),
                })
            })
            .collect()
    }

    fn resolve_apply_stages(
        &mut self,
        apply: &Apply,
        target: &ResolvedApplyTarget,
    ) -> Vec<ResolvedApplyStage> {
        apply
            .stages
            .iter()
            .filter(|stage| stage.is_executable())
            .filter_map(|stage| self.resolve_apply_stage(apply.record_range, stage, target))
            .collect()
    }

    fn resolve_apply_stage(
        &mut self,
        owner: TimeRange,
        stage: &ApplyStage,
        target: &ResolvedApplyTarget,
    ) -> Option<ResolvedApplyStage> {
        let stage_range = apply_ranges::absolute(owner, stage.active_range)?;
        if !target_ranges(target).any(|range| apply_ranges::intersect(range, stage_range).is_some())
        {
            return None;
        }
        let (active_range, operation) = match &stage.operation {
            ApplyOperation::Color { pipeline } => (
                stage_range,
                ResolvedApplyOperation::Color {
                    pipeline: self.resolve_color_pipeline(pipeline)?,
                },
            ),
            ApplyOperation::Effect { effect } if effect.enabled => (
                stage_range,
                ResolvedApplyOperation::Effect {
                    effect_type: effect.effect_type.clone(),
                    parameters: effect.parameters.clone(),
                },
            ),
            ApplyOperation::Effect { .. } => return None,
        };
        Some(ResolvedApplyStage {
            id: stage.id.clone(),
            active_range,
            operation,
        })
    }
}

fn target_ranges(target: &ResolvedApplyTarget) -> impl Iterator<Item = TimeRange> + '_ {
    match target {
        ResolvedApplyTarget::CompositeBand { active_ranges, .. }
        | ResolvedApplyTarget::Layer { active_ranges, .. } => {
            Either::Ranges(active_ranges.iter().copied())
        }
        ResolvedApplyTarget::ItemSet { items } => {
            Either::Items(items.iter().map(|item| item.active_range))
        }
    }
}

enum Either<A, B> {
    Ranges(A),
    Items(B),
}

impl<T, A: Iterator<Item = T>, B: Iterator<Item = T>> Iterator for Either<A, B> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Ranges(values) => values.next(),
            Self::Items(values) => values.next(),
        }
    }
}
