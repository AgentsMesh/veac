use super::*;

#[test]
fn split_interpolates_and_reidentifies_the_animated_crop_viewport() {
    let mut project = sample_project();
    crop_mut(&mut project).replace(curve(0, 600));
    let result = applied(apply_edit_batch(
        &project,
        &batch(
            "op_split_crop_curve",
            &project,
            vec![EditOperation::SplitClip {
                clip_id: ItemId::new("itm_video").unwrap(),
                at: time(300),
                right_clip_id: ItemId::new("itm_crop_right").unwrap(),
                relation_fragments: vec![],
            }],
        ),
    ));
    let clips = &result.project.sequences[0].tracks[0].clips;
    let left = crop_keys(&clips[0]);
    let right = crop_keys(&clips[1]);
    assert_eq!(left.last().unwrap().value.x, 0.25);
    assert_eq!(left.last().unwrap().value.width, 0.75);
    assert_eq!(right.first().unwrap().value, left.last().unwrap().value);
    assert_eq!(right.first().unwrap().time, time(0));
    assert!(left
        .iter()
        .all(|left| right.iter().all(|right| left.id != right.id)));
}

#[test]
fn crop_keyframes_participate_in_global_move_remove_and_change_tracking() {
    let mut project = sample_project();
    crop_mut(&mut project).replace(curve(0, 20));
    let id = ItemId::new("itm_video").unwrap();
    let moved = applied(apply_edit_batch(
        &project,
        &batch(
            "op_move_crop_key",
            &project,
            vec![EditOperation::EditKeyframe {
                edit: KeyframeEdit::Move {
                    clip_id: id.clone(),
                    keyframe_id: KeyframeId::new("kf_crop_a").unwrap(),
                    time: time(5),
                },
            }],
        ),
    ));
    assert_eq!(
        crop_keys(&moved.project.sequences[0].tracks[0].clips[0])[0].time,
        time(5)
    );
    let removed = apply_edit_batch(
        &moved,
        &batch(
            "op_remove_crop_key",
            &moved,
            vec![EditOperation::EditKeyframe {
                edit: KeyframeEdit::Remove {
                    clip_id: id,
                    keyframe_id: KeyframeId::new("kf_crop_b").unwrap(),
                },
            }],
        ),
    );
    let EditOutcome::Applied {
        project,
        changed_objects,
        ..
    } = removed
    else {
        panic!("crop key removal must apply")
    };
    assert_eq!(
        crop_keys(&project.project.sequences[0].tracks[0].clips[0]).len(),
        1
    );
    assert!(changed_objects.contains(&ChangedObjectId::Keyframe {
        id: KeyframeId::new("kf_crop_b").unwrap(),
    }));
}

fn crop_mut(project: &mut ProjectEnvelope) -> &mut Option<Animatable<Rect>> {
    &mut project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .transform
        .crop
}

fn crop_keys(clip: &Clip) -> &[Keyframe<Rect>] {
    clip.visual
        .as_ref()
        .unwrap()
        .transform
        .crop
        .as_ref()
        .unwrap()
        .keyframes()
        .unwrap()
}

fn curve(start: i64, end: i64) -> Animatable<Rect> {
    Animatable::Keyframes {
        keyframes: vec![
            crop_key("kf_crop_a", start, 0.0, 1.0),
            crop_key("kf_crop_b", end, 0.5, 0.5),
        ],
    }
}

fn crop_key(id: &str, at: i64, x: f64, width: f64) -> Keyframe<Rect> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value: Rect {
            x,
            y: 0.0,
            width,
            height: 1.0,
        },
        interpolation: Interpolation::Linear,
    }
}
