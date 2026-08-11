mod support;

use tempfile::tempdir;
use veac_evidence::*;

#[test]
fn prepared_bundle_contains_authoritative_and_human_derivatives() {
    let suite = support::suite();
    let (validated, plan, provenance) = support::plan_and_provenance(&suite);
    let run = ObservationRun {
        observations: support::observations(),
        failures: vec![],
    };
    let bundle = prepare_evidence_bundle(&validated, &plan, &run, &provenance).unwrap();
    assert_eq!(bundle.manifest.outcome, EvidenceOutcome::Pass);
    assert_eq!(
        bundle.manifest.cache_key,
        evidence_cache_key(&provenance).unwrap()
    );
    for path in [
        "results.json",
        "results.csv",
        "junit.xml",
        "index.html",
        "observations.json",
        "contact-sheet.png",
        "frames/actual.png",
    ] {
        assert!(
            bundle.artifacts.iter().any(|value| value.path == path),
            "{path}"
        );
    }
    let results = bytes(&bundle, "results.json");
    let report: EvidenceReportV1 = serde_json::from_slice(results).unwrap();
    assert_eq!(report.outcome, EvidenceOutcome::Pass);
    assert!(bytes(&bundle, "contact-sheet.png").starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(std::str::from_utf8(bytes(&bundle, "junit.xml"))
        .unwrap()
        .contains("testsuite"));
}

#[test]
fn publication_is_atomic_and_cache_reuses_verified_contents() {
    let suite = support::suite();
    let (validated, plan, provenance) = support::plan_and_provenance(&suite);
    let bundle = prepare_evidence_bundle(
        &validated,
        &plan,
        &ObservationRun {
            observations: support::observations(),
            failures: vec![],
        },
        &provenance,
    )
    .unwrap();
    let temp = tempdir().unwrap();
    let delivery = temp.path().join("delivery/evidence");
    publish_evidence_bundle(&bundle, &delivery).unwrap();
    assert!(delivery.join("bundle.json").is_file());
    assert!(publish_evidence_bundle(&bundle, &delivery).is_err());

    let cache = EvidenceCache::new(temp.path().join("cache"));
    let first = cache.publish(&bundle).unwrap();
    let second = cache.publish(&bundle).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        cache.lookup(&bundle.manifest.cache_key).unwrap(),
        Some(first.clone())
    );
    std::fs::write(first.root.join("results.csv"), b"corrupt").unwrap();
    assert!(cache.lookup(&bundle.manifest.cache_key).is_err());
}

#[test]
fn contract_mismatches_and_error_outcomes_are_not_cached() {
    let suite = support::suite();
    let (validated, plan, mut provenance) = support::plan_and_provenance(&suite);
    provenance.suite_sha256 = "f".repeat(64);
    let run = ObservationRun {
        observations: support::observations(),
        failures: vec![],
    };
    assert!(matches!(
        prepare_evidence_bundle(&validated, &plan, &run, &provenance),
        Err(BundleError::InvalidContract(_))
    ));

    let (_, _, provenance) = support::plan_and_provenance(&suite);
    let mut bundle = prepare_evidence_bundle(&validated, &plan, &run, &provenance).unwrap();
    bundle.manifest.outcome = EvidenceOutcome::Error;
    let cache = EvidenceCache::new(tempdir().unwrap().path());
    assert!(cache.publish(&bundle).is_err());
    assert!(cache.lookup("bad").is_err());
}

fn bytes<'a>(bundle: &'a PreparedEvidenceBundle, path: &str) -> &'a [u8] {
    &bundle
        .artifacts
        .iter()
        .find(|value| value.path == path)
        .unwrap()
        .bytes
}
