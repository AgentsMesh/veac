use crate::{
    motion_deceleration, reveal_prefix, AssertionKind, AssertionResult, EvidenceSuiteV1,
    MotionProfileSpec, ObservationSet, RevealOrderSpec,
};

use super::{error, region, result, status};

pub(super) fn reveal(
    value: &RevealOrderSpec,
    suite: &EvidenceSuiteV1,
    set: &ObservationSet,
) -> AssertionResult {
    let Some(baseline) = set.frames.get(&value.baseline_sample_id) else {
        return error(
            &value.id,
            AssertionKind::RevealOrder,
            "baseline sample is missing",
        );
    };
    let checkpoints = match value
        .checkpoints
        .iter()
        .map(|checkpoint| {
            set.frames
                .get(&checkpoint.sample_id)
                .map(|frame| (checkpoint, frame))
        })
        .collect::<Option<Vec<_>>>()
    {
        Some(values) => values,
        None => {
            return error(
                &value.id,
                AssertionKind::RevealOrder,
                "checkpoint sample is missing",
            )
        }
    };
    let regions = value
        .subjects
        .iter()
        .filter_map(|subject| {
            suite
                .regions
                .iter()
                .find(|region| region.id == subject.region_id)
        })
        .collect::<Vec<_>>();
    let stats = match reveal_prefix(
        baseline,
        &checkpoints,
        &regions,
        value.change_threshold,
        value.visible_minimum,
        value.hidden_maximum,
    ) {
        Ok(stats) => stats,
        Err(problem) => return error(&value.id, AssertionKind::RevealOrder, problem.to_string()),
    };
    let mut output = result(
        &value.id,
        AssertionKind::RevealOrder,
        status(stats.matches_expected_prefix),
        "reveal prefix evaluated",
    );
    for (checkpoint, row) in value.checkpoints.iter().zip(stats.change_fractions) {
        output
            .series
            .insert(format!("checkpoint.{}", checkpoint.sample_id), row);
    }
    output
}

pub(super) fn motion(
    value: &MotionProfileSpec,
    suite: &EvidenceSuiteV1,
    set: &ObservationSet,
) -> AssertionResult {
    let frames = match value
        .sample_ids
        .iter()
        .map(|id| set.frames.get(id))
        .collect::<Option<Vec<_>>>()
    {
        Some(values) => values,
        None => {
            return error(
                &value.id,
                AssertionKind::MotionProfile,
                "motion sample is missing",
            )
        }
    };
    let stats = match motion_deceleration(
        &frames,
        region(suite, &value.region_id),
        value.metric,
        value.minimum_interval_motion,
        value.minimum_deceleration_ratio,
        value.monotonic_tolerance,
    ) {
        Ok(stats) => stats,
        Err(problem) => return error(&value.id, AssertionKind::MotionProfile, problem.to_string()),
    };
    let passed = stats.monotonically_decelerating && stats.passes_minimums;
    let mut output = result(
        &value.id,
        AssertionKind::MotionProfile,
        status(passed),
        "motion profile evaluated",
    );
    output
        .metrics
        .insert("first_to_last_ratio".into(), stats.first_to_last_ratio);
    output
        .series
        .insert("interval_motion".into(), stats.interval_motion);
    output
}
