mod support;

use veac_evidence::*;

#[test]
fn public_discriminants_and_error_messages_are_exercised() {
    let suite = support::suite();
    let expected = [
        AssertionKind::DecodeComplete,
        AssertionKind::Alpha,
        AssertionKind::PixelDiff,
        AssertionKind::Bounds,
        AssertionKind::LayerOrder,
        AssertionKind::CompositeOver,
        AssertionKind::RevealOrder,
        AssertionKind::MotionProfile,
    ];
    for (assertion, kind) in suite.assertions.iter().zip(expected) {
        assert!(!assertion.id().is_empty());
        assert_eq!(assertion.kind(), kind);
    }
    assert_eq!(validate(suite).unwrap().into_suite().id, "acceptance");

    for error in [
        MetricError::InvalidFrame("bad".into()),
        MetricError::MissingAlpha,
        MetricError::GeometryMismatch,
        MetricError::RegionOutsideFrame,
        MetricError::EmptySelection,
        MetricError::InvalidThreshold,
    ] {
        assert!(!error.to_string().is_empty());
    }
    for error in [
        BundleError::Encode("bad".into()),
        BundleError::UnsafePath("bad".into()),
        BundleError::DuplicatePath("bad".into()),
        BundleError::NonFiniteMetric("bad".into()),
        BundleError::InvalidContract("bad".into()),
        BundleError::Io(std::io::Error::other("bad")),
    ] {
        assert!(!error.to_string().is_empty());
    }
    let payload = ObservationError::InvalidPayload {
        expected: 4,
        actual: 3,
    };
    assert!(payload.to_string().contains("3 bytes"));
    assert!(ObservationError::InvalidDimensions
        .to_string()
        .contains("invalid"));
}

#[test]
fn rgb_pixels_missing_references_and_empty_composites_are_explicit() {
    let black = support::rgb(&[[0, 0, 0]; 16]);
    let white = support::rgb(&[[255, 255, 255]; 16]);
    assert_eq!(
        diff_stats(&black, &white, None, DiffChannels::Rgba, 1)
            .unwrap()
            .changed_fraction,
        1.0
    );
    assert_eq!(
        mask(
            &support::actual(),
            None,
            None,
            &MaskSpec::Difference {
                reference_sample_id: "missing".into(),
                minimum_delta: 1,
            },
        )
        .unwrap_err(),
        MetricError::EmptySelection
    );
    let transparent = support::solid([0, 0, 0, 0]);
    assert_eq!(
        compare_composite(
            &support::actual(),
            &support::underlay(),
            &transparent,
            None,
            1,
        )
        .unwrap_err(),
        MetricError::EmptySelection
    );
}

#[test]
fn zero_last_motion_and_invalid_profile_parameters_are_defined() {
    let first = support::motion(0);
    let second = support::motion(8);
    let frames = [&first, &second, &second];
    let result = motion_deceleration(
        &frames,
        None,
        MotionMetric::ChangedFraction { threshold: 1 },
        0.0,
        1.0,
        0.0,
    )
    .unwrap();
    assert_eq!(result.first_to_last_ratio, f64::MAX);
    assert!(!result.passes_minimums || result.monotonically_decelerating);
    for parameters in [(-1.0, 1.0, 0.0), (0.0, f64::NAN, 0.0), (0.0, 1.0, -1.0)] {
        assert_eq!(
            motion_deceleration(
                &frames,
                None,
                MotionMetric::ChangedFraction { threshold: 1 },
                parameters.0,
                parameters.1,
                parameters.2,
            )
            .unwrap_err(),
            MetricError::InvalidThreshold
        );
    }
}

#[test]
fn all_source_bindings_remain_structured() {
    let mut suite = support::suite();
    suite.sources.push(SourceSpec {
        id: "input".into(),
        binding: SourceBinding::BoundInput {
            input_id: "fixture".into(),
        },
    });
    assert!(validate(suite).is_ok());
}
