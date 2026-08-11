use std::collections::BTreeSet;

use crate::{AssertionSpec, EvidenceSuiteV1, MaskSpec, SourceBinding};

use super::{check_id, issue, ValidationIssue};

pub(super) fn validate(suite: &EvidenceSuiteV1, issues: &mut Vec<ValidationIssue>) {
    let sources = ids(suite.sources.iter().map(|value| value.id.as_str()));
    let samples = ids(suite.samples.iter().map(|value| value.id.as_str()));
    let regions = ids(suite.regions.iter().map(|value| value.id.as_str()));
    for (index, source) in suite.sources.iter().enumerate() {
        let target = match &source.binding {
            SourceBinding::Deliverable { deliverable_id } => deliverable_id,
            SourceBinding::Artifact { artifact_id } => artifact_id,
            SourceBinding::BoundInput { input_id } => input_id,
        };
        check_id(issues, &format!("sources[{index}].binding"), target);
    }
    for (index, sample) in suite.samples.iter().enumerate() {
        reference(
            issues,
            &sources,
            &format!("samples[{index}].source_id"),
            &sample.source_id,
        );
        if sample.at.value < 0 || !sample.at.is_valid() {
            issue(
                issues,
                &format!("samples[{index}].at"),
                "sample time must be non-negative",
            );
        }
    }
    for (index, assertion) in suite.assertions.iter().enumerate() {
        let path = format!("assertions[{index}]");
        validate_assertion(assertion, &path, &sources, &samples, &regions, issues);
    }
}

fn validate_assertion(
    value: &AssertionSpec,
    path: &str,
    sources: &BTreeSet<&str>,
    samples: &BTreeSet<&str>,
    regions: &BTreeSet<&str>,
    issues: &mut Vec<ValidationIssue>,
) {
    let sample = |issues: &mut Vec<_>, field: &str, value: &str| {
        reference(issues, samples, &format!("{path}.{field}"), value)
    };
    let region = |issues: &mut Vec<_>, value: &Option<String>| {
        if let Some(value) = value {
            reference(issues, regions, &format!("{path}.region_id"), value);
        }
    };
    match value {
        AssertionSpec::DecodeComplete(spec) => reference(
            issues,
            sources,
            &format!("{path}.source_id"),
            &spec.source_id,
        ),
        AssertionSpec::Alpha(spec) => {
            sample(issues, "sample_id", &spec.sample_id);
            region(issues, &spec.region_id);
        }
        AssertionSpec::PixelDiff(spec) => {
            sample(issues, "left_sample_id", &spec.left_sample_id);
            sample(issues, "right_sample_id", &spec.right_sample_id);
            region(issues, &spec.region_id);
        }
        AssertionSpec::Bounds(spec) => {
            sample(issues, "sample_id", &spec.sample_id);
            region(issues, &spec.region_id);
            if let MaskSpec::Difference {
                reference_sample_id,
                ..
            } = &spec.mask
            {
                sample(issues, "mask.reference_sample_id", reference_sample_id);
            }
        }
        AssertionSpec::LayerOrder(_) => {}
        AssertionSpec::CompositeOver(spec) => {
            sample(issues, "actual_sample_id", &spec.actual_sample_id);
            sample(issues, "underlay_sample_id", &spec.underlay_sample_id);
            sample(issues, "overlay_sample_id", &spec.overlay_sample_id);
            region(issues, &spec.region_id);
        }
        AssertionSpec::RevealOrder(spec) => {
            sample(issues, "baseline_sample_id", &spec.baseline_sample_id);
            for subject in &spec.subjects {
                reference(
                    issues,
                    regions,
                    &format!("{path}.subjects.region_id"),
                    &subject.region_id,
                );
            }
            for checkpoint in &spec.checkpoints {
                sample(issues, "checkpoints.sample_id", &checkpoint.sample_id);
            }
        }
        AssertionSpec::MotionProfile(spec) => {
            for sample_id in &spec.sample_ids {
                sample(issues, "sample_ids", sample_id);
            }
            region(issues, &spec.region_id);
        }
    }
}

fn ids<'a>(values: impl Iterator<Item = &'a str>) -> BTreeSet<&'a str> {
    values.collect()
}

fn reference(
    issues: &mut Vec<ValidationIssue>,
    available: &BTreeSet<&str>,
    path: &str,
    value: &str,
) {
    if !available.contains(value) {
        issue(issues, path, "reference does not exist");
    }
}
