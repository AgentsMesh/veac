mod support;

use veac_evidence::*;

#[test]
fn frame_contract_and_regions_are_exact() {
    let frame = support::underlay();
    assert!(frame.validate().is_ok());
    assert_eq!(PixelFormat::Rgb8.channels(), 3);
    assert_eq!(PixelFormat::Rgba8.channels(), 4);
    let full = resolve_region(&frame, None).unwrap();
    assert_eq!((full.x, full.y, full.width, full.height), (0, 0, 4, 4));
    let suite = support::suite();
    assert_eq!(
        resolve_region(&frame, Some(&suite.regions[0]))
            .unwrap()
            .width,
        2
    );
    assert_eq!(
        resolve_region(&frame, Some(&suite.regions[1])).unwrap().x,
        2
    );

    let mut invalid = frame.clone();
    invalid.data.pop();
    assert!(matches!(
        invalid.validate(),
        Err(ObservationError::InvalidPayload { .. })
    ));
    invalid.width = 0;
    assert_eq!(
        invalid.validate().unwrap_err(),
        ObservationError::InvalidDimensions
    );
    let outside = RegionSpec {
        id: "bad".into(),
        space: RegionSpace::Pixels {
            x: 4,
            y: 0,
            width: 1,
            height: 1,
        },
    };
    assert_eq!(
        resolve_region(&frame, Some(&outside)).unwrap_err(),
        MetricError::RegionOutsideFrame
    );
}

#[test]
fn alpha_and_diff_metrics_cover_rgb_rgba_and_alpha() {
    let overlay = support::overlay();
    let stats = alpha_stats(&overlay, None, 0, 255).unwrap();
    assert_eq!((stats.minimum, stats.maximum), (0, 255));
    assert_eq!(stats.transparent_fraction, 0.75);
    assert_eq!(stats.opaque_fraction, 0.25);
    assert_eq!(stats.partial_fraction, 0.0);
    assert_eq!(
        alpha_stats(&overlay, None, 200, 100).unwrap_err(),
        MetricError::InvalidThreshold
    );
    let rgb = support::rgb(&[[0, 0, 0]; 16]);
    assert_eq!(
        alpha_stats(&rgb, None, 0, 255).unwrap_err(),
        MetricError::MissingAlpha
    );

    let actual = support::actual();
    let under = support::underlay();
    for channels in [DiffChannels::Rgb, DiffChannels::Rgba, DiffChannels::Alpha] {
        let diff = diff_stats(&under, &actual, None, channels, 1).unwrap();
        assert!(diff.rmse >= 0.0 && diff.mae >= 0.0 && diff.maximum_delta >= 0.0);
    }
    assert_eq!(
        diff_stats(
            &under,
            &FrameObservation { width: 3, ..actual },
            None,
            DiffChannels::Rgb,
            1
        )
        .unwrap_err(),
        MetricError::GeometryMismatch
    );
}

#[test]
fn masks_bounds_and_composites_have_explicit_empty_behavior() {
    let overlay = support::overlay();
    let alpha_mask = mask(&overlay, None, None, &MaskSpec::Alpha { minimum: 1 }).unwrap();
    let box_value = bounds(&alpha_mask, 4, 4).unwrap();
    assert_eq!(
        (box_value.x, box_value.y, box_value.width, box_value.height),
        (1, 1, 2, 2)
    );
    let difference = mask(
        &support::actual(),
        Some(&support::underlay()),
        None,
        &MaskSpec::Difference {
            reference_sample_id: "under".into(),
            minimum_delta: 1,
        },
    )
    .unwrap();
    assert_eq!(bounds(&difference, 4, 4), Some(box_value));
    let dark = mask(
        &support::underlay(),
        None,
        None,
        &MaskSpec::Luma {
            threshold: 20,
            above: false,
        },
    )
    .unwrap();
    assert_eq!(dark.values.iter().filter(|value| **value).count(), 16);
    let none = mask(&overlay, None, None, &MaskSpec::Alpha { minimum: 255 }).unwrap();
    assert!(bounds(
        &Mask {
            values: vec![false; 16],
            ..none
        },
        4,
        4
    )
    .is_none());

    assert_eq!(source_over([0, 0, 0, 0], [0, 0, 0, 0]), [0, 0, 0, 0]);
    let stats =
        compare_composite(&support::actual(), &support::underlay(), &overlay, None, 1).unwrap();
    assert_eq!(stats.expected_rmse, 0.0);
    assert_eq!(stats.improvement_ratio, f64::MAX);
    assert_eq!(
        compare_composite(
            &support::actual(),
            &support::underlay(),
            &support::rgb(&[[0, 0, 0]; 16]),
            None,
            1
        )
        .unwrap_err(),
        MetricError::MissingAlpha
    );
}
