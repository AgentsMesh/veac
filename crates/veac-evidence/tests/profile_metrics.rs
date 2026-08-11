mod support;

use veac_evidence::*;

#[test]
fn reveal_prefix_uses_visible_and_hidden_hysteresis() {
    let suite = support::suite();
    let observations = support::observations();
    let baseline = &observations.frames["under"];
    let checkpoints = [
        (
            &RevealCheckpoint {
                sample_id: "reveal1".into(),
                visible_prefix: 1,
            },
            &observations.frames["reveal1"],
        ),
        (
            &RevealCheckpoint {
                sample_id: "reveal2".into(),
                visible_prefix: 2,
            },
            &observations.frames["reveal2"],
        ),
    ];
    let regions = [&suite.regions[0], &suite.regions[1]];
    let result = reveal_prefix(baseline, &checkpoints, &regions, 1, 0.9, 0.1).unwrap();
    assert!(result.matches_expected_prefix);
    assert_eq!(
        result.change_fractions,
        vec![vec![1.0, 0.0], vec![1.0, 1.0]]
    );

    let wrong = [(
        &RevealCheckpoint {
            sample_id: "reveal2".into(),
            visible_prefix: 1,
        },
        &observations.frames["reveal2"],
    )];
    assert!(
        !reveal_prefix(baseline, &wrong, &regions, 1, 0.9, 0.1)
            .unwrap()
            .matches_expected_prefix
    );
    assert_eq!(
        reveal_prefix(baseline, &[], &regions, 1, 0.9, 0.1).unwrap_err(),
        MetricError::InvalidThreshold
    );
}

#[test]
fn changed_fraction_motion_proves_deceleration() {
    let observations = support::observations();
    let frames = [
        &observations.frames["motion0"],
        &observations.frames["motion1"],
        &observations.frames["motion2"],
    ];
    let stats = motion_deceleration(
        &frames,
        None,
        MotionMetric::ChangedFraction { threshold: 1 },
        0.2,
        2.0,
        0.0,
    )
    .unwrap();
    assert_eq!(stats.interval_motion, vec![0.75, 0.25]);
    assert_eq!(stats.first_to_last_ratio, 3.0);
    assert!(stats.monotonically_decelerating && stats.passes_minimums);
    assert_eq!(
        motion_deceleration(
            &frames[..2],
            None,
            MotionMetric::ChangedFraction { threshold: 1 },
            0.0,
            1.0,
            0.0,
        )
        .unwrap_err(),
        MetricError::InvalidThreshold
    );
}

#[test]
fn alpha_centroid_motion_is_weighted_and_reports_empty_masks() {
    let frames = [alpha_point(0), alpha_point(2), alpha_point(3)];
    let refs = frames.iter().collect::<Vec<_>>();
    let stats = motion_deceleration(
        &refs,
        None,
        MotionMetric::AlphaCentroid { minimum: 1 },
        0.5,
        2.0,
        0.0,
    )
    .unwrap();
    assert_eq!(stats.interval_motion, vec![2.0, 1.0]);
    assert!(stats.monotonically_decelerating && stats.passes_minimums);

    let empty = support::solid([0, 0, 0, 0]);
    let empties = [&empty, &empty, &empty];
    assert_eq!(
        motion_deceleration(
            &empties,
            None,
            MotionMetric::AlphaCentroid { minimum: 1 },
            0.0,
            1.0,
            0.0,
        )
        .unwrap_err(),
        MetricError::EmptySelection
    );
}

fn alpha_point(x: usize) -> FrameObservation {
    let mut pixels = [[0, 0, 0, 0]; 16];
    pixels[x] = [255, 255, 255, 255];
    support::rgba(&pixels)
}
