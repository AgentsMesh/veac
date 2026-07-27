use super::*;
use crate::test_support::range;

#[test]
fn rejects_crossing_composite_bands_during_the_same_time() {
    let mut project = four_layer_project();
    project.project.sequences[0].applies = vec![
        apply(
            "apl_left",
            "aps_left",
            band("trk_video", "trk_layer_b"),
            0,
            300,
        ),
        apply(
            "apl_right",
            "aps_right",
            band("trk_layer_a", "trk_layer_c"),
            0,
            300,
        ),
    ];
    assert_code(&validation_codes(&project), "APPLY_BAND_CROSSING");
}

#[test]
fn allows_nested_and_time_disjoint_composite_bands() {
    let mut project = four_layer_project();
    project.project.sequences[0].applies = vec![
        apply(
            "apl_outer",
            "aps_outer",
            band("trk_video", "trk_layer_c"),
            0,
            300,
        ),
        apply(
            "apl_inner",
            "aps_inner",
            band("trk_layer_a", "trk_layer_b"),
            0,
            300,
        ),
        apply(
            "apl_late",
            "aps_late",
            band("trk_layer_a", "trk_layer_c"),
            300,
            300,
        ),
    ];
    assert!(validate(&project).is_ok());
}

fn four_layer_project() -> ProjectEnvelope {
    let mut project = sample_project();
    project.project.sequences[0].tracks[1].order = 4;
    for (track_id, item_id, order) in [
        ("trk_layer_a", "itm_layer_a", 1),
        ("trk_layer_b", "itm_layer_b", 2),
        ("trk_layer_c", "itm_layer_c", 3),
    ] {
        let mut track = project.project.sequences[0].tracks[0].clone();
        track.id = TrackId::new(track_id).unwrap();
        track.order = order;
        track.clips[0].id = ItemId::new(item_id).unwrap();
        track.clips[0].visual = Some(crate::test_support::identity_layout_visual());
        track.clips[0].effects.clear();
        project.project.sequences[0].tracks.push(track);
    }
    project
}

fn band(from: &str, through: &str) -> ApplyTarget {
    ApplyTarget::CompositeBand {
        from_track_id: TrackId::new(from).unwrap(),
        through_track_id: TrackId::new(through).unwrap(),
    }
}

fn apply(id: &str, stage: &str, target: ApplyTarget, start: i64, duration: i64) -> Apply {
    Apply {
        id: ApplyId::new(id).unwrap(),
        enabled: true,
        record_range: range(start, duration),
        target,
        stages: vec![ApplyStage {
            id: ApplyStageId::new(stage).unwrap(),
            enabled: true,
            active_range: None,
            operation: ApplyOperation::Color {
                pipeline: super::apply_target_tests::empty_pipeline(),
            },
        }],
        mix: ApplyMix::default(),
    }
}
