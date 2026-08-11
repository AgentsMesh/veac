mod support;

use veac_evidence::*;

struct Broken;

impl serde::Serialize for Broken {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(<S::Error as serde::ser::Error>::custom("broken fixture"))
    }
}

#[test]
fn canonical_json_and_bundle_manifest_are_content_bound_and_sorted() {
    let suite = support::suite();
    let report = evaluate(&validate(suite.clone()).unwrap(), &support::observations());
    let (_, _, provenance) = support::plan_and_provenance(&suite);
    let manifest = build_bundle_manifest(
        &suite,
        &report,
        &provenance,
        vec![
            BundleArtifactContent {
                path: "z/frame.png".into(),
                bytes: vec![2, 3],
            },
            BundleArtifactContent {
                path: "a/results.csv".into(),
                bytes: vec![1],
            },
        ],
    )
    .unwrap();
    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.outcome, EvidenceOutcome::Pass);
    assert_eq!(manifest.artifacts[0].path, "a/results.csv");
    assert_eq!(manifest.artifacts[0].size_bytes, 1);
    assert_eq!(manifest.artifacts[0].sha256, sha256_hex(&[1]));
    assert_eq!(
        canonical_json(&manifest).unwrap(),
        canonical_json(&manifest).unwrap()
    );
    assert_eq!(manifest.suite_sha256.len(), 64);
    assert_eq!(manifest.report_sha256.len(), 64);
}

#[test]
fn bundle_paths_are_relative_normal_and_unique() {
    let suite = support::suite();
    let report = evaluate(&validate(suite.clone()).unwrap(), &support::observations());
    let (_, _, provenance) = support::plan_and_provenance(&suite);
    for path in ["", "/absolute", "a/../b", "./a"] {
        let error = build_bundle_manifest(
            &suite,
            &report,
            &provenance,
            vec![BundleArtifactContent {
                path: path.into(),
                bytes: vec![],
            }],
        )
        .unwrap_err();
        assert!(matches!(error, BundleError::UnsafePath(_)));
        assert!(error.to_string().contains("unsafe"));
    }
    let duplicate = build_bundle_manifest(
        &suite,
        &report,
        &provenance,
        vec![
            BundleArtifactContent {
                path: "same".into(),
                bytes: vec![1],
            },
            BundleArtifactContent {
                path: "same".into(),
                bytes: vec![2],
            },
        ],
    )
    .unwrap_err();
    assert!(matches!(duplicate, BundleError::DuplicatePath(_)));
}

#[test]
fn bundle_rejects_non_finite_metrics() {
    let mut values = std::collections::BTreeMap::new();
    values.insert("bad", f64::NAN);
    assert_eq!(canonical_json(&values).unwrap(), br#"{"bad":null}"#);

    let suite = support::suite();
    let mut report = evaluate(&validate(suite.clone()).unwrap(), &support::observations());
    let (_, _, provenance) = support::plan_and_provenance(&suite);
    report.assertions[0].metrics.insert("bad".into(), f64::NAN);
    assert!(matches!(
        build_bundle_manifest(&suite, &report, &provenance, vec![]),
        Err(BundleError::NonFiniteMetric(_))
    ));
    assert!(matches!(
        canonical_json(&Broken),
        Err(BundleError::Encode(_))
    ));
}
