use crate::{
    alpha_stats, diff_stats, AlphaSpec, AssertionKind, AssertionResult, DecodeCompleteSpec,
    DiffExpectation, ObservationSet, PixelDiffSpec, RangeExpectation,
};

use super::{error, region, result, status};

pub(super) fn decode(value: &DecodeCompleteSpec, set: &ObservationSet) -> AssertionResult {
    let Some(observed) = set.decodes.get(&value.source_id) else {
        return error(
            &value.id,
            AssertionKind::DecodeComplete,
            "decode observation is missing",
        );
    };
    let passed = observed.complete
        && observed.errors.is_empty()
        && observed.decoded_frames >= value.minimum_frames;
    let mut output = result(
        &value.id,
        AssertionKind::DecodeComplete,
        status(passed),
        if passed {
            "decode is complete"
        } else {
            "decode contract failed"
        },
    );
    output
        .metrics
        .insert("decoded_frames".into(), observed.decoded_frames as f64);
    output
}

pub(super) fn alpha(
    value: &AlphaSpec,
    suite: &crate::EvidenceSuiteV1,
    set: &ObservationSet,
) -> AssertionResult {
    let Some(frame) = set.frames.get(&value.sample_id) else {
        return error(&value.id, AssertionKind::Alpha, "sample frame is missing");
    };
    let stats = match alpha_stats(
        frame,
        region(suite, &value.region_id),
        value.expectation.transparent_below,
        value.expectation.opaque_above,
    ) {
        Ok(value) => value,
        Err(problem) => return error(&value.id, AssertionKind::Alpha, problem.to_string()),
    };
    let passed = within(stats.mean, value.expectation.mean)
        && within(
            stats.transparent_fraction,
            value.expectation.transparent_fraction,
        )
        && within(stats.partial_fraction, value.expectation.partial_fraction)
        && within(stats.opaque_fraction, value.expectation.opaque_fraction);
    let mut output = result(
        &value.id,
        AssertionKind::Alpha,
        status(passed),
        "alpha metrics evaluated",
    );
    output.metrics.extend([
        ("minimum".into(), f64::from(stats.minimum) / 255.0),
        ("maximum".into(), f64::from(stats.maximum) / 255.0),
        ("mean".into(), stats.mean),
        ("transparent_fraction".into(), stats.transparent_fraction),
        ("partial_fraction".into(), stats.partial_fraction),
        ("opaque_fraction".into(), stats.opaque_fraction),
    ]);
    output
}

pub(super) fn diff(
    value: &PixelDiffSpec,
    suite: &crate::EvidenceSuiteV1,
    set: &ObservationSet,
) -> AssertionResult {
    let Some(left) = set.frames.get(&value.left_sample_id) else {
        return error(
            &value.id,
            AssertionKind::PixelDiff,
            "left sample is missing",
        );
    };
    let Some(right) = set.frames.get(&value.right_sample_id) else {
        return error(
            &value.id,
            AssertionKind::PixelDiff,
            "right sample is missing",
        );
    };
    let stats = match diff_stats(
        left,
        right,
        region(suite, &value.region_id),
        value.channels,
        value.change_threshold,
    ) {
        Ok(value) => value,
        Err(problem) => return error(&value.id, AssertionKind::PixelDiff, problem.to_string()),
    };
    let passed = diff_within(stats, &value.expectation);
    let mut output = result(
        &value.id,
        AssertionKind::PixelDiff,
        status(passed),
        "pixel diff evaluated",
    );
    output.metrics.extend([
        ("rmse".into(), stats.rmse),
        ("mae".into(), stats.mae),
        ("maximum_delta".into(), stats.maximum_delta),
        ("changed_fraction".into(), stats.changed_fraction),
    ]);
    output
}

fn within(value: f64, range: Option<RangeExpectation>) -> bool {
    range.is_none_or(|range| {
        range.minimum.is_none_or(|minimum| value >= minimum)
            && range.maximum.is_none_or(|maximum| value <= maximum)
    })
}

fn diff_within(value: crate::DiffStats, expected: &DiffExpectation) -> bool {
    within(value.rmse, expected.rmse)
        && within(value.mae, expected.mae)
        && within(value.maximum_delta, expected.maximum_delta)
        && within(value.changed_fraction, expected.changed_fraction)
}
