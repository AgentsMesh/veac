use crate::{
    compare_composite, AssertionKind, AssertionResult, CompositeOverSpec, EvidenceSuiteV1,
    ObservationSet,
};

use super::{error, region, result, status};

pub(super) fn evaluate(
    value: &CompositeOverSpec,
    suite: &EvidenceSuiteV1,
    set: &ObservationSet,
) -> AssertionResult {
    let Some(actual) = set.frames.get(&value.actual_sample_id) else {
        return error(
            &value.id,
            AssertionKind::CompositeOver,
            "actual sample is missing",
        );
    };
    let Some(underlay) = set.frames.get(&value.underlay_sample_id) else {
        return error(
            &value.id,
            AssertionKind::CompositeOver,
            "underlay sample is missing",
        );
    };
    let Some(overlay) = set.frames.get(&value.overlay_sample_id) else {
        return error(
            &value.id,
            AssertionKind::CompositeOver,
            "overlay sample is missing",
        );
    };
    let stats = match compare_composite(
        actual,
        underlay,
        overlay,
        region(suite, &value.region_id),
        value.overlay_alpha_minimum,
    ) {
        Ok(stats) => stats,
        Err(problem) => return error(&value.id, AssertionKind::CompositeOver, problem.to_string()),
    };
    let passed = stats.expected_rmse <= value.maximum_expected_rmse
        && stats.improvement_ratio >= value.minimum_improvement_ratio;
    let mut output = result(
        &value.id,
        AssertionKind::CompositeOver,
        status(passed),
        "source-over composite evaluated",
    );
    output.metrics.extend([
        ("expected_rmse".into(), stats.expected_rmse),
        ("underlay_rmse".into(), stats.underlay_rmse),
        ("improvement_ratio".into(), stats.improvement_ratio),
        ("selected_fraction".into(), stats.selected_fraction),
    ]);
    output
}
