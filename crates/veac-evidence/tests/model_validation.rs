mod support;

use veac_evidence::*;

#[test]
fn strict_suite_round_trips_and_exposes_a_schema() {
    let suite = support::suite();
    let json = serde_json::to_string(&suite).unwrap();
    assert_eq!(
        serde_json::from_str::<EvidenceSuiteV1>(&json).unwrap(),
        suite
    );
    assert!(serde_json::from_str::<EvidenceSuiteV1>(&json.replacen(
        "\"id\":\"acceptance\"",
        "\"id\":\"acceptance\",\"extra\":1",
        1
    ))
    .is_err());
    let schema = schemars::schema_for!(EvidenceSuiteV1);
    assert_eq!(
        schema.get("title").and_then(|value| value.as_str()),
        Some("EvidenceSuiteV1")
    );
    assert!(validate(suite).is_ok());
}

#[test]
fn core_validation_rejects_versions_ids_duplicates_limits_and_geometry() {
    let mut suite = support::suite();
    suite.schema_version = 2;
    suite.id = "Bad".into();
    suite.sources.push(suite.sources[0].clone());
    suite.samples[0].at.timescale = 0;
    suite.samples[0].source_id = "missing".into();
    suite.regions[0].space = RegionSpace::Normalized {
        x: 0.9,
        y: 0.0,
        width: 0.2,
        height: 1.0,
    };
    suite.regions[1].space = RegionSpace::Pixels {
        x: 0,
        y: 0,
        width: 0,
        height: 1,
    };
    let error = validate(suite).unwrap_err();
    assert!(error.issues.len() >= 7);
    assert!(error.to_string().contains("validation error"));

    let mut oversized = support::suite();
    for index in 0..513 {
        oversized.sources.push(SourceSpec {
            id: format!("source{index}"),
            binding: SourceBinding::Artifact {
                artifact_id: "artifact".into(),
            },
        });
    }
    assert!(validate(oversized)
        .unwrap_err()
        .issues
        .iter()
        .any(|value| value.message.contains("budget")));
}

#[test]
fn assertion_validation_is_closed_and_fail_fast_at_execution_boundaries() {
    let mut suite = support::suite();
    for assertion in &mut suite.assertions {
        match assertion {
            AssertionSpec::DecodeComplete(value) => value.minimum_frames = 0,
            AssertionSpec::Alpha(value) => {
                value.expectation.transparent_below = 200;
                value.expectation.opaque_above = 100;
                value.expectation.mean = Some(RangeExpectation {
                    minimum: None,
                    maximum: None,
                });
            }
            AssertionSpec::PixelDiff(value) => {
                value.right_sample_id = value.left_sample_id.clone();
                value.expectation = DiffExpectation {
                    rmse: None,
                    mae: None,
                    maximum_delta: None,
                    changed_fraction: None,
                };
            }
            AssertionSpec::Bounds(value) => value.mask = MaskSpec::Alpha { minimum: 0 },
            AssertionSpec::LayerOrder(value) => value.lower_entity = value.upper_entity.clone(),
            AssertionSpec::CompositeOver(value) => {
                value.overlay_sample_id = value.actual_sample_id.clone();
                value.overlay_alpha_minimum = 0;
                value.minimum_improvement_ratio = 0.5;
            }
            AssertionSpec::RevealOrder(value) => {
                value.subjects[1].id = value.subjects[0].id.clone();
                value.hidden_maximum = value.visible_minimum;
                value.checkpoints[0].visible_prefix = 9;
            }
            AssertionSpec::MotionProfile(value) => {
                value.sample_ids = vec!["motion0".into(), "motion0".into()];
                value.minimum_deceleration_ratio = 0.5;
            }
        }
    }
    let error = validate(suite).unwrap_err();
    assert!(error.issues.len() >= 12, "{:?}", error.issues);
}

#[test]
fn every_reference_family_is_checked() {
    let mut suite = support::suite();
    for assertion in &mut suite.assertions {
        match assertion {
            AssertionSpec::Bounds(value) => {
                value.mask = MaskSpec::Difference {
                    reference_sample_id: "missing".into(),
                    minimum_delta: 1,
                }
            }
            AssertionSpec::RevealOrder(value) => value.subjects[0].region_id = "missing".into(),
            AssertionSpec::MotionProfile(value) => value.region_id = Some("missing".into()),
            _ => {}
        }
    }
    assert!(
        validate(suite)
            .unwrap_err()
            .issues
            .iter()
            .filter(|value| value.message.contains("reference"))
            .count()
            >= 3
    );
}
