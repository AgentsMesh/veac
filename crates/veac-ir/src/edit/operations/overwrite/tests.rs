use crate::test_support::{sample_project, time};
use crate::*;

#[test]
fn spanning_overwrite_preserves_forward_and_reverse_source_handles() {
    for (direction, expected_left, expected_right, operation_id) in [
        (PlaybackDirection::Forward, 100, 500, "op_forward_span"),
        (PlaybackDirection::Reverse, 500, 100, "op_reverse_span"),
    ] {
        let mut project = prepared_project();
        set_linear(
            &mut project.project.sequences[0].tracks[0].clips[0],
            time(100),
            direction,
        );
        let result = applied(apply_edit_batch(
            &project,
            &overwrite(&project, operation_id, time(200), time(200)),
        ));
        let clips = &result.project.sequences[0].tracks[0].clips;
        assert_eq!(clips.len(), 3);
        assert_eq!(clips[0].record_range, range(0, 200));
        assert_eq!(linear_start(&clips[0]), time(expected_left));
        assert_eq!(clips[2].record_range, range(400, 200));
        assert_eq!(linear_start(&clips[2]), time(expected_right));
    }
}

#[test]
fn spanning_overwrite_crops_source_curves_across_the_removed_interval() {
    let mut project = prepared_project();
    project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![
                segment(300, 0, 300, SourceTimeInterpolation::Linear),
                segment(100, 300, 300, SourceTimeInterpolation::Hold),
                segment(200, 300, 500, SourceTimeInterpolation::Linear),
            ],
        },
        frame_synthesis: FrameSynthesisPolicy::Blend,
        out_of_range: SourceOutOfRangePolicy::Strict,
    });
    let result = applied(apply_edit_batch(
        &project,
        &overwrite(&project, "op_curve_span", time(250), time(200)),
    ));
    let clips = &result.project.sequences[0].tracks[0].clips;
    let left = curve_segments(&clips[0]);
    let right = curve_segments(&clips[2]);
    assert_eq!(
        (left[0].source_start, left[0].source_end),
        (time(0), time(250))
    );
    assert_eq!(right[0].record_duration, time(150));
    assert_eq!(
        (right[0].source_start, right[0].source_end),
        (time(350), time(500))
    );
}

#[test]
fn overwrite_rejects_existing_ids_and_overflowed_ranges() {
    let project = prepared_project();
    let mut existing = replacement(&project);
    existing.id = ItemId::new("itm_video").unwrap();
    let mut edit = overwrite(&project, "op_existing", time(200), time(200));
    let EditOperation::OverwriteClip { clip, .. } = &mut edit.operations[0] else {
        unreachable!()
    };
    **clip = existing;
    assert!(matches!(
        apply_edit_batch(&project, &edit),
        EditOutcome::Rejected { .. }
    ));

    let maximum = crate::MAX_SAFE_INTEGER as i64;
    let overflow = overwrite(&project, "op_overflow", time(maximum), time(1));
    assert!(matches!(
        apply_edit_batch(&project, &overflow),
        EditOutcome::Rejected { .. }
    ));
}

fn prepared_project() -> ProjectEnvelope {
    let mut project = sample_project();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.effects.clear();
    clip.visual.as_mut().unwrap().opacity = Animatable::constant(1.0);
    project
}

fn overwrite(
    project: &ProjectEnvelope,
    operation_id: &str,
    start: RationalTime,
    duration: RationalTime,
) -> EditBatch {
    let mut clip = replacement(project);
    clip.record_range = TimeRange { start, duration };
    EditBatch {
        operation_id: OperationId::new(operation_id).unwrap(),
        base_revision: project.project.revision,
        atomic: true,
        preconditions: vec![],
        operations: vec![EditOperation::OverwriteClip {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_video").unwrap(),
            clip: Box::new(clip),
            split_fragments: vec![OverwriteFragment {
                source_clip_id: ItemId::new("itm_video").unwrap(),
                right_fragment_id: ItemId::new("itm_video_right").unwrap(),
                relation_fragments: vec![],
            }],
        }],
    }
}

fn replacement(project: &ProjectEnvelope) -> Clip {
    let mut clip = project.project.sequences[0].tracks[0].clips[0].clone();
    clip.id = ItemId::new("itm_replacement").unwrap();
    clip.source_mapping = Some(SourceMapping::linear(time(0), Rational::new(1, 1).unwrap()));
    clip
}

fn set_linear(clip: &mut Clip, start: RationalTime, direction: PlaybackDirection) {
    let SourceTimeMap::Linear {
        source_start,
        direction: current,
        ..
    } = &mut clip.source_mapping.as_mut().unwrap().time_map
    else {
        panic!("linear mapping")
    };
    *source_start = start;
    *current = direction;
}

fn linear_start(clip: &Clip) -> RationalTime {
    let SourceTimeMap::Linear { source_start, .. } =
        &clip.source_mapping.as_ref().unwrap().time_map
    else {
        panic!("linear mapping")
    };
    *source_start
}

fn curve_segments(clip: &Clip) -> &[SourceTimeSegment] {
    let SourceTimeMap::Curve { segments } = &clip.source_mapping.as_ref().unwrap().time_map else {
        panic!("curve mapping")
    };
    segments
}

fn segment(
    duration: i64,
    start: i64,
    end: i64,
    kind: SourceTimeInterpolation,
) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(start),
        source_end: time(end),
        interpolation: kind,
    }
}

fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange {
        start: time(start),
        duration: time(duration),
    }
}

fn applied(outcome: EditOutcome) -> ProjectEnvelope {
    let EditOutcome::Applied { project, .. } = outcome else {
        panic!("overwrite should apply")
    };
    project
}
