use veac_evidence::*;

pub fn suite() -> EvidenceSuiteV1 {
    EvidenceSuiteV1 {
        schema_version: 1,
        id: "acceptance".into(),
        sources: vec![SourceSpec {
            id: "final".into(),
            binding: SourceBinding::Deliverable {
                deliverable_id: "master".into(),
            },
        }],
        samples: sample_ids()
            .into_iter()
            .enumerate()
            .map(|(index, id)| SampleSpec {
                id: id.into(),
                source_id: "final".into(),
                at: RationalTime {
                    value: index as i64,
                    timescale: 30,
                },
            })
            .collect(),
        regions: vec![
            RegionSpec {
                id: "left".into(),
                space: RegionSpace::Normalized {
                    x: 0.0,
                    y: 0.0,
                    width: 0.5,
                    height: 1.0,
                },
            },
            RegionSpec {
                id: "right".into(),
                space: RegionSpace::Pixels {
                    x: 2,
                    y: 0,
                    width: 2,
                    height: 4,
                },
            },
        ],
        assertions: assertions(),
    }
}

fn sample_ids() -> [&'static str; 8] {
    [
        "under", "overlay", "actual", "reveal1", "reveal2", "motion0", "motion1", "motion2",
    ]
}

fn assertions() -> Vec<AssertionSpec> {
    vec![
        AssertionSpec::DecodeComplete(DecodeCompleteSpec {
            id: "decode".into(),
            source_id: "final".into(),
            minimum_frames: 1,
        }),
        AssertionSpec::Alpha(AlphaSpec {
            id: "alpha".into(),
            sample_id: "overlay".into(),
            region_id: None,
            expectation: AlphaExpectation {
                transparent_below: 0,
                opaque_above: 255,
                mean: Some(range(0.2, 0.3)),
                transparent_fraction: Some(range(0.7, 0.8)),
                partial_fraction: Some(range(0.0, 0.0)),
                opaque_fraction: Some(range(0.2, 0.3)),
            },
        }),
        AssertionSpec::PixelDiff(PixelDiffSpec {
            id: "diff".into(),
            left_sample_id: "under".into(),
            right_sample_id: "actual".into(),
            region_id: None,
            channels: DiffChannels::Rgb,
            change_threshold: 1,
            expectation: DiffExpectation {
                rmse: Some(range(0.2, 0.6)),
                mae: None,
                maximum_delta: None,
                changed_fraction: Some(range(0.2, 0.3)),
            },
        }),
        bounds_assertion(),
        AssertionSpec::LayerOrder(LayerOrderSpec {
            id: "layer".into(),
            upper_entity: "copy".into(),
            lower_entity: "picture".into(),
        }),
        composite_assertion(),
        reveal_assertion(),
        motion_assertion(),
    ]
}

fn bounds_assertion() -> AssertionSpec {
    AssertionSpec::Bounds(BoundsSpec {
        id: "bounds".into(),
        sample_id: "overlay".into(),
        region_id: None,
        mask: MaskSpec::Alpha { minimum: 1 },
        expectation: BoundsExpectation {
            non_empty: true,
            minimum_margin_pixels: Some(1),
            maximum_width_pixels: Some(2),
            maximum_height_pixels: Some(2),
        },
    })
}

fn composite_assertion() -> AssertionSpec {
    AssertionSpec::CompositeOver(CompositeOverSpec {
        id: "composite".into(),
        actual_sample_id: "actual".into(),
        underlay_sample_id: "under".into(),
        overlay_sample_id: "overlay".into(),
        region_id: None,
        overlay_alpha_minimum: 1,
        maximum_expected_rmse: 0.0,
        minimum_improvement_ratio: 2.0,
    })
}

fn reveal_assertion() -> AssertionSpec {
    AssertionSpec::RevealOrder(RevealOrderSpec {
        id: "reveal".into(),
        baseline_sample_id: "under".into(),
        subjects: vec![
            RevealSubject {
                id: "a".into(),
                region_id: "left".into(),
            },
            RevealSubject {
                id: "b".into(),
                region_id: "right".into(),
            },
        ],
        checkpoints: vec![
            RevealCheckpoint {
                sample_id: "reveal1".into(),
                visible_prefix: 1,
            },
            RevealCheckpoint {
                sample_id: "reveal2".into(),
                visible_prefix: 2,
            },
        ],
        change_threshold: 1,
        visible_minimum: 0.9,
        hidden_maximum: 0.1,
    })
}

fn motion_assertion() -> AssertionSpec {
    AssertionSpec::MotionProfile(MotionProfileSpec {
        id: "motion".into(),
        sample_ids: vec!["motion0".into(), "motion1".into(), "motion2".into()],
        region_id: None,
        metric: MotionMetric::ChangedFraction { threshold: 1 },
        minimum_interval_motion: 0.2,
        minimum_deceleration_ratio: 2.0,
        monotonic_tolerance: 0.0,
    })
}

fn range(minimum: f64, maximum: f64) -> RangeExpectation {
    RangeExpectation {
        minimum: Some(minimum),
        maximum: Some(maximum),
    }
}
