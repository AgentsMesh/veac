use crate::{
    AssertionResult, AssertionSpec, AssertionStatus, EvidenceOutcome, EvidenceReportV1,
    ObservationSet, ValidatedSuite, EVIDENCE_REPORT_SCHEMA_VERSION,
};

mod basic;
mod composite;
mod geometry;
mod reveal_motion;

pub fn evaluate(suite: &ValidatedSuite, observations: &ObservationSet) -> EvidenceReportV1 {
    let suite = suite.as_suite();
    let assertions = suite
        .assertions
        .iter()
        .map(|assertion| evaluate_one(assertion, suite, observations))
        .collect::<Vec<_>>();
    let outcome = if assertions
        .iter()
        .any(|value| value.status == AssertionStatus::Error)
    {
        EvidenceOutcome::Error
    } else if assertions
        .iter()
        .any(|value| value.status == AssertionStatus::Fail)
    {
        EvidenceOutcome::Fail
    } else {
        EvidenceOutcome::Pass
    };
    EvidenceReportV1 {
        schema_version: EVIDENCE_REPORT_SCHEMA_VERSION,
        suite_id: suite.id.clone(),
        outcome,
        assertions,
    }
}

fn evaluate_one(
    assertion: &AssertionSpec,
    suite: &crate::EvidenceSuiteV1,
    observations: &ObservationSet,
) -> AssertionResult {
    match assertion {
        AssertionSpec::DecodeComplete(value) => basic::decode(value, observations),
        AssertionSpec::Alpha(value) => basic::alpha(value, suite, observations),
        AssertionSpec::PixelDiff(value) => basic::diff(value, suite, observations),
        AssertionSpec::Bounds(value) => geometry::bounds(value, suite, observations),
        AssertionSpec::LayerOrder(value) => geometry::layer(value, observations),
        AssertionSpec::CompositeOver(value) => composite::evaluate(value, suite, observations),
        AssertionSpec::RevealOrder(value) => reveal_motion::reveal(value, suite, observations),
        AssertionSpec::MotionProfile(value) => reveal_motion::motion(value, suite, observations),
    }
}

pub(super) fn result(
    id: &str,
    kind: crate::AssertionKind,
    status: AssertionStatus,
    message: impl Into<String>,
) -> AssertionResult {
    AssertionResult {
        id: id.to_owned(),
        kind,
        status,
        message: message.into(),
        metrics: Default::default(),
        series: Default::default(),
    }
}

pub(super) fn error(
    id: &str,
    kind: crate::AssertionKind,
    message: impl Into<String>,
) -> AssertionResult {
    result(id, kind, AssertionStatus::Error, message)
}

pub(super) fn status(passed: bool) -> AssertionStatus {
    if passed {
        AssertionStatus::Pass
    } else {
        AssertionStatus::Fail
    }
}

pub(super) fn region<'a>(
    suite: &'a crate::EvidenceSuiteV1,
    id: &Option<String>,
) -> Option<&'a crate::RegionSpec> {
    id.as_ref()
        .and_then(|id| suite.regions.iter().find(|value| value.id == *id))
}
