use std::collections::BTreeSet;

use crate::{AssertionSpec, DiffExpectation, EvidenceSuiteV1, MaskSpec, RangeExpectation};

use super::{check_id, issue, ValidationIssue};

pub(super) fn validate(suite: &EvidenceSuiteV1, issues: &mut Vec<ValidationIssue>) {
    for (index, assertion) in suite.assertions.iter().enumerate() {
        let path = format!("assertions[{index}]");
        match assertion {
            AssertionSpec::DecodeComplete(value) => {
                if value.minimum_frames == 0 {
                    issue(issues, &path, "minimum_frames must be positive");
                }
            }
            AssertionSpec::Alpha(value) => {
                if value.expectation.transparent_below >= value.expectation.opaque_above {
                    issue(issues, &path, "alpha thresholds overlap");
                }
                for range in [
                    value.expectation.mean,
                    value.expectation.transparent_fraction,
                    value.expectation.partial_fraction,
                    value.expectation.opaque_fraction,
                ] {
                    check_range(range, true, &path, issues);
                }
            }
            AssertionSpec::PixelDiff(value) => {
                if value.left_sample_id == value.right_sample_id {
                    issue(issues, &path, "pixel diff samples must be different");
                }
                check_diff(&value.expectation, &path, issues);
            }
            AssertionSpec::Bounds(value) => {
                if matches!(
                    value.mask,
                    MaskSpec::Alpha { minimum: 0 }
                        | MaskSpec::Difference {
                            minimum_delta: 0,
                            ..
                        }
                ) {
                    issue(issues, &path, "mask threshold must be positive");
                }
            }
            AssertionSpec::LayerOrder(value) => {
                if value.upper_entity.is_empty()
                    || value.lower_entity.is_empty()
                    || value.upper_entity == value.lower_entity
                {
                    issue(
                        issues,
                        &path,
                        "layer entities must be distinct and non-empty",
                    );
                }
            }
            AssertionSpec::CompositeOver(value) => {
                let samples = [
                    value.actual_sample_id.as_str(),
                    value.underlay_sample_id.as_str(),
                    value.overlay_sample_id.as_str(),
                ];
                if samples.into_iter().collect::<BTreeSet<_>>().len() != 3 {
                    issue(issues, &path, "composite samples must be distinct");
                }
                if value.overlay_alpha_minimum == 0
                    || !non_negative(value.maximum_expected_rmse)
                    || !value.minimum_improvement_ratio.is_finite()
                    || value.minimum_improvement_ratio < 1.0
                {
                    issue(issues, &path, "invalid composite thresholds");
                }
            }
            AssertionSpec::RevealOrder(value) => check_reveal(value, &path, issues),
            AssertionSpec::MotionProfile(value) => {
                if value.sample_ids.len() < 3
                    || value.sample_ids.iter().collect::<BTreeSet<_>>().len()
                        != value.sample_ids.len()
                    || !non_negative(value.minimum_interval_motion)
                    || !value.minimum_deceleration_ratio.is_finite()
                    || value.minimum_deceleration_ratio < 1.0
                    || !non_negative(value.monotonic_tolerance)
                {
                    issue(issues, &path, "invalid motion profile");
                }
            }
        }
    }
}

fn check_diff(value: &DiffExpectation, path: &str, issues: &mut Vec<ValidationIssue>) {
    let ranges = [
        value.rmse,
        value.mae,
        value.maximum_delta,
        value.changed_fraction,
    ];
    if ranges.iter().all(Option::is_none) {
        issue(issues, path, "pixel diff requires at least one expectation");
    }
    for range in ranges {
        check_range(range, true, path, issues);
    }
}

fn check_reveal(value: &crate::RevealOrderSpec, path: &str, issues: &mut Vec<ValidationIssue>) {
    if value.subjects.is_empty()
        || value.checkpoints.is_empty()
        || !unit(value.hidden_maximum)
        || !unit(value.visible_minimum)
        || value.hidden_maximum >= value.visible_minimum
    {
        issue(issues, path, "invalid reveal thresholds or empty matrix");
    }
    let mut subject_ids = BTreeSet::new();
    for subject in &value.subjects {
        check_id(issues, path, &subject.id);
        if !subject_ids.insert(subject.id.as_str()) {
            issue(issues, path, "reveal subject ids must be unique");
        }
    }
    let mut previous = 0;
    for checkpoint in &value.checkpoints {
        if checkpoint.visible_prefix < previous || checkpoint.visible_prefix > value.subjects.len()
        {
            issue(
                issues,
                path,
                "visible prefixes must be monotonic and in range",
            );
        }
        previous = checkpoint.visible_prefix;
    }
}

fn check_range(
    range: Option<RangeExpectation>,
    unit_interval: bool,
    path: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(range) = range else { return };
    let minimum = range.minimum.unwrap_or(0.0);
    let maximum = range.maximum.unwrap_or(1.0);
    let valid = minimum.is_finite()
        && maximum.is_finite()
        && minimum <= maximum
        && (!unit_interval || minimum >= 0.0 && maximum <= 1.0);
    if !valid || range.minimum.is_none() && range.maximum.is_none() {
        issue(issues, path, "invalid metric range");
    }
}

fn non_negative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn unit(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
