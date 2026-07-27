use super::*;

#[test]
fn split_reslices_ramp_and_hold_with_continuous_source_boundaries() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(curve());
    let edit = batch(
        "op_split_time_curve",
        &project,
        vec![EditOperation::SplitClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            at: time(350),
            right_clip_id: ItemId::new("itm_curve_right").unwrap(),
            relation_fragments: vec![],
        }],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let clips = &result.project.sequences[0].tracks[0].clips;
    let left = segments(&clips[0]);
    let right = segments(&clips[1]);
    assert_eq!(durations(left), vec![time(300), time(50)]);
    assert_eq!(durations(right), vec![time(50), time(200)]);
    assert_eq!(left.last().unwrap().source_end, time(600));
    assert_eq!(right.first().unwrap().source_start, time(600));
    assert_eq!(right[0].interpolation, SourceTimeInterpolation::Hold);
}

#[test]
fn trim_slip_and_roll_crop_extrapolate_and_shift_curves_exactly() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(curve());
    let trim = batch(
        "op_trim_time_curve",
        &project,
        vec![EditOperation::TrimClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            edge: TrimEdge::In,
            delta: time(150),
            ripple: true,
        }],
    );
    let trimmed = applied(apply_edit_batch(&project, &trim));
    let trimmed_segments = segments(&trimmed.project.sequences[0].tracks[0].clips[0]);
    assert_eq!(trimmed_segments[0].source_start, time(300));
    assert_eq!(
        durations(trimmed_segments),
        vec![time(150), time(100), time(200)]
    );

    let slip = batch(
        "op_slip_time_curve",
        &project,
        vec![EditOperation::SlipClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            source_delta: time(25),
        }],
    );
    let slipped = applied(apply_edit_batch(&project, &slip));
    let shifted = segments(&slipped.project.sequences[0].tracks[0].clips[0]);
    assert_eq!(shifted[0].source_start, time(25));
    assert_eq!(shifted.last().unwrap().source_end, time(725));

    let mut adjacent = magnetic_project();
    for clip in &mut adjacent.project.sequences[0].tracks[0].clips {
        clip.source_mapping = Some(single_curve(linear_start(clip).value));
    }
    let roll = batch(
        "op_roll_time_curve",
        &adjacent,
        vec![EditOperation::RollEdit {
            left_clip_id: ItemId::new("itm_video").unwrap(),
            right_clip_id: ItemId::new("itm_middle").unwrap(),
            delta: time(60),
        }],
    );
    let rolled = applied(apply_edit_batch(&adjacent, &roll));
    let clips = &rolled.project.sequences[0].tracks[0].clips;
    assert_eq!(segments(&clips[0]).last().unwrap().source_end, time(660));
    assert_eq!(segments(&clips[1])[0].source_start, time(660));
}

#[test]
fn typed_source_mapping_edit_rejects_non_media_and_applies_atomically() {
    let project = sample_project();
    let apply = batch(
        "op_set_time_curve",
        &project,
        vec![EditOperation::SetSourceMapping {
            clip_id: ItemId::new("itm_video").unwrap(),
            source_mapping: curve(),
        }],
    );
    let changed = applied(apply_edit_batch(&project, &apply));
    assert!(matches!(
        segments(&changed.project.sequences[0].tracks[0].clips[0])[1].interpolation,
        SourceTimeInterpolation::Hold
    ));
    let reject = batch(
        "op_set_non_media_curve",
        &project,
        vec![EditOperation::SetSourceMapping {
            clip_id: ItemId::new("itm_caption").unwrap(),
            source_mapping: curve(),
        }],
    );
    assert_rejected(apply_edit_batch(&project, &reject), "EDIT_REJECTED");
}

fn curve() -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![
                segment(300, 0, 600, false),
                segment(100, 600, 600, true),
                segment(200, 600, 700, false),
            ],
        },
        frame_synthesis: FrameSynthesisPolicy::Blend,
        out_of_range: SourceOutOfRangePolicy::Strict,
    }
}

fn single_curve(start: i64) -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![segment(600, start, start + 600, false)],
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    }
}

fn segment(duration: i64, start: i64, end: i64, hold: bool) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(start),
        source_end: time(end),
        interpolation: if hold {
            SourceTimeInterpolation::Hold
        } else {
            SourceTimeInterpolation::Linear
        },
    }
}

fn segments(clip: &Clip) -> &[SourceTimeSegment] {
    let SourceTimeMap::Curve { segments } = &clip.source_mapping.as_ref().unwrap().time_map else {
        panic!("curve")
    };
    segments
}

fn durations(segments: &[SourceTimeSegment]) -> Vec<RationalTime> {
    segments
        .iter()
        .map(|segment| segment.record_duration)
        .collect()
}
