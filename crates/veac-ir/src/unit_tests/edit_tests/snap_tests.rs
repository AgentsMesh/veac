use super::*;

#[test]
fn snap_candidates_combine_edges_and_exact_frame_grid_with_stable_ordering() {
    let project = sample_project();
    let request = SnapRequest {
        sequence_id: SequenceId::new("seq_main").unwrap(),
        time: time(294),
        tolerance: time(10),
        include_clip_edges: true,
        include_frame_grid: true,
        excluded_items: vec![],
    };
    let candidates = snap_candidates(&project, &request).unwrap();
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].time, time(300));
    assert_eq!(candidates[0].distance, time(6));
    assert!(matches!(
        &candidates[0].target,
        SnapTarget::ClipEnd { item_id, .. } if item_id.as_str() == "itm_caption"
    ));
    assert_eq!(
        candidates[1].target,
        SnapTarget::FrameGrid { frame_index: 15 }
    );

    let excluded = SnapRequest {
        excluded_items: vec![ItemId::new("itm_caption").unwrap()],
        ..request
    };
    let candidates = snap_candidates(&project, &excluded).unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(matches!(candidates[0].target, SnapTarget::FrameGrid { .. }));
}

#[test]
fn snap_ties_are_sorted_by_time_target_kind_and_typed_ids() {
    let project = sample_project();
    let request = SnapRequest {
        sequence_id: SequenceId::new("seq_main").unwrap(),
        time: time(0),
        tolerance: time(0),
        include_clip_edges: true,
        include_frame_grid: true,
        excluded_items: vec![],
    };
    let candidates = snap_candidates(&project, &request).unwrap();
    assert_eq!(candidates.len(), 3);
    let targets: Vec<_> = candidates.iter().map(|value| &value.target).collect();
    assert!(matches!(
        targets[0],
        SnapTarget::ClipStart { track_id, .. } if track_id.as_str() == "trk_captions"
    ));
    assert!(matches!(
        targets[1],
        SnapTarget::ClipStart { track_id, .. } if track_id.as_str() == "trk_video"
    ));
    assert_eq!(targets[2], &SnapTarget::FrameGrid { frame_index: 0 });
}

#[test]
fn snap_rejects_invalid_requests_projects_sequences_and_inexact_frame_grids() {
    let project = sample_project();
    let base = SnapRequest {
        sequence_id: SequenceId::new("seq_main").unwrap(),
        time: time(10),
        tolerance: time(5),
        include_clip_edges: false,
        include_frame_grid: true,
        excluded_items: vec![],
    };
    let invalid = SnapRequest {
        tolerance: time(-1),
        ..base.clone()
    };
    assert_eq!(
        snap_candidates(&project, &invalid).unwrap_err().code,
        "INVALID_SNAP_REQUEST"
    );
    let missing = SnapRequest {
        sequence_id: SequenceId::new("seq_missing").unwrap(),
        ..base.clone()
    };
    assert_eq!(
        snap_candidates(&project, &missing).unwrap_err().code,
        "INVALID_SNAP_REQUEST"
    );
    let mut inexact = project.clone();
    inexact.project.sequences[0].settings.frame_rate = Rational::new(30_000, 1001).unwrap();
    assert_eq!(
        snap_candidates(&inexact, &base).unwrap_err().code,
        "INEXACT_FRAME_GRID"
    );
    let mut invalid_project = project.clone();
    invalid_project.project.timebase = 0;
    assert_eq!(
        snap_candidates(&invalid_project, &base).unwrap_err().code,
        "TIMEBASE"
    );
}

#[test]
fn snap_can_disable_both_sources_and_round_trip_as_protocol_data() {
    let project = sample_project();
    let request = SnapRequest {
        sequence_id: SequenceId::new("seq_main").unwrap(),
        time: time(10),
        tolerance: time(20),
        include_clip_edges: false,
        include_frame_grid: false,
        excluded_items: vec![],
    };
    assert!(snap_candidates(&project, &request).unwrap().is_empty());
    let json = serde_json::to_string(&request).unwrap();
    assert_eq!(serde_json::from_str::<SnapRequest>(&json).unwrap(), request);
}
