mod support;

use veac_evidence::*;

#[test]
fn complete_observations_produce_a_deterministic_pass_report() {
    let suite = validate(support::suite()).unwrap();
    let observations = support::observations();
    let first = evaluate(&suite, &observations);
    let second = evaluate(&suite, &observations);
    assert_eq!(first, second);
    assert_eq!(first.outcome, EvidenceOutcome::Pass);
    assert_eq!(first.assertions.len(), 8);
    assert!(first
        .assertions
        .iter()
        .all(|value| value.status == AssertionStatus::Pass));
    assert!(first
        .assertions
        .iter()
        .all(|value| !value.metrics.is_empty() || !value.series.is_empty()));
    let schema = schemars::schema_for!(EvidenceReportV1);
    assert_eq!(
        schema.get("title").and_then(|value| value.as_str()),
        Some("EvidenceReportV1")
    );
}

#[test]
fn predicate_failures_are_distinct_from_missing_observations() {
    let suite = validate(support::suite()).unwrap();
    let mut failed = support::observations();
    failed.decodes.get_mut("final").unwrap().complete = false;
    failed.layer_orders.get_mut("layer").unwrap().coactive = false;
    failed.frames.insert("actual".into(), support::underlay());
    failed.frames.insert("reveal1".into(), support::underlay());
    failed.frames.insert("motion1".into(), support::motion(4));
    let report = evaluate(&suite, &failed);
    assert_eq!(report.outcome, EvidenceOutcome::Fail);
    assert!(
        report
            .assertions
            .iter()
            .filter(|value| value.status == AssertionStatus::Fail)
            .count()
            >= 4
    );

    let missing = evaluate(&suite, &ObservationSet::default());
    assert_eq!(missing.outcome, EvidenceOutcome::Error);
    assert!(missing
        .assertions
        .iter()
        .all(|value| value.status == AssertionStatus::Error));
}

#[test]
fn each_observation_family_reports_a_stable_error() {
    let suite = validate(support::suite()).unwrap();
    let cases = [
        ("final", "decode"),
        ("overlay", "alpha"),
        ("under", "diff"),
        ("actual", "composite"),
        ("reveal2", "reveal"),
        ("motion2", "motion"),
    ];
    for (missing, assertion_id) in cases {
        let mut observations = support::observations();
        if missing == "final" {
            observations.decodes.remove(missing);
        } else {
            observations.frames.remove(missing);
        }
        let report = evaluate(&suite, &observations);
        let result = report
            .assertions
            .iter()
            .find(|value| value.id == assertion_id)
            .unwrap();
        assert_eq!(result.status, AssertionStatus::Error, "{missing}");
    }
    let mut observations = support::observations();
    observations.layer_orders.clear();
    let report = evaluate(&suite, &observations);
    assert_eq!(
        report
            .assertions
            .iter()
            .find(|value| value.id == "layer")
            .unwrap()
            .status,
        AssertionStatus::Error
    );
}
