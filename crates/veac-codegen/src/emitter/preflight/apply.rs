use std::collections::BTreeSet;

use veac_plan::canonical::{ApplyId, ApplyStageId, RationalTime, TimeRange};
use veac_plan::{ResolvedApply, ResolvedApplyOperation, ResolvedRenderPlan, ResolvedSequence};

use super::{apply_target, color, composition, effects, Check};

pub(super) fn validate(plan: &ResolvedRenderPlan, check: &mut Check) {
    let mut apply_ids = BTreeSet::new();
    let mut stage_ids = BTreeSet::new();
    for sequence in &plan.sequences {
        let mut orders = BTreeSet::new();
        for apply in &sequence.applies {
            let identity_valid = ApplyId::new(apply.id.to_string()).is_ok()
                && apply_ids.insert(apply.id.clone())
                && orders.insert(apply.source_order);
            if !identity_valid {
                check.push(
                    "PLAN_APPLY_ID_INVALID",
                    Some(apply.id.to_string()),
                    "apply IDs and source orders must be valid and unique",
                );
            }
            validate_one(plan, check, sequence, apply, &mut stage_ids);
        }
        validate_overlap(check, sequence);
    }
}

fn validate_one(
    plan: &ResolvedRenderPlan,
    check: &mut Check,
    sequence: &ResolvedSequence,
    apply: &ResolvedApply,
    stage_ids: &mut BTreeSet<ApplyStageId>,
) {
    let sequence_range = TimeRange {
        start: RationalTime {
            value: 0,
            timescale: plan.header.source.timebase,
        },
        duration: sequence.duration,
    };
    if apply.record_range.start.timescale != plan.header.source.timebase
        || !within(sequence_range, apply.record_range)
        || apply.stages.is_empty()
    {
        check.push(
            "PLAN_APPLY_RANGE_INVALID",
            Some(apply.id.to_string()),
            "apply range, stages, or opacity is invalid",
        );
    }
    apply_target::validate(check, sequence, apply);
    composition::validate_masks(check, &apply.id.to_string(), &apply.mix.masks);
    for stage in &apply.stages {
        if ApplyStageId::new(stage.id.to_string()).is_err()
            || !stage_ids.insert(stage.id.clone())
            || stage.active_range.start.timescale != plan.header.source.timebase
            || !within(apply.record_range, stage.active_range)
        {
            check.push(
                "PLAN_APPLY_STAGE_INVALID",
                Some(apply.id.to_string()),
                "apply stage ID and active range must be valid and unique",
            );
        }
        match &stage.operation {
            ResolvedApplyOperation::Color { pipeline } => {
                color::validate_pipeline(check, plan, &apply.id.to_string(), pipeline);
            }
            ResolvedApplyOperation::Effect {
                effect_type,
                parameters,
            } => effects::validate_apply_effect(
                check,
                &apply.id.to_string(),
                effect_type,
                parameters,
            ),
        }
    }
}

fn validate_overlap(check: &mut Check, sequence: &ResolvedSequence) {
    for (index, left) in sequence.applies.iter().enumerate() {
        if !matches!(
            left.target,
            veac_plan::ResolvedApplyTarget::CompositeBand { .. }
        ) {
            continue;
        }
        for right in sequence.applies.iter().skip(index + 1) {
            if !matches!(
                right.target,
                veac_plan::ResolvedApplyTarget::CompositeBand { .. }
            ) || !overlap(left.record_range, right.record_range)
            {
                continue;
            }
            let Some((a, b)) = apply_target::interval(sequence, left) else {
                continue;
            };
            let Some((c, d)) = apply_target::interval(sequence, right) else {
                continue;
            };
            if (a < c && c <= b && b < d) || (c < a && a <= d && d < b) {
                check.push(
                    "PLAN_APPLY_BANDS_CROSS",
                    Some(right.id.to_string()),
                    "time-overlapping composite bands must be laminar",
                );
            }
        }
    }
}

fn within(owner: TimeRange, child: TimeRange) -> bool {
    child.duration.value > 0 && owner.start <= child.start && child.end().ok() <= owner.end().ok()
}

fn overlap(left: TimeRange, right: TimeRange) -> bool {
    left.start < right.end().expect("preflight timebase")
        && right.start < left.end().expect("preflight timebase")
}
