use super::*;

#[test]
fn upsert_supports_every_typed_curve_target_and_reports_children() {
    let mut project = sample_project();
    let clip_id = ItemId::new("itm_video").unwrap();
    project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .transform
        .crop = Some(Animatable::constant(Rect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    }));
    let keys = [
        number_key("kf_rotation_edit", 10, 15.0),
        number_key("kf_opacity_edit", 30, 0.5),
        number_key("kf_gain_edit", 0, 0.8),
        number_key("kf_pan_edit", 0, 0.2),
        number_key("kf_brightness_edit", 0, 0.3),
    ];
    let operations = vec![
        keyframe(KeyframeEdit::UpsertNumber {
            clip_id: clip_id.clone(),
            target: NumberCurveTarget::VisualRotationDegrees,
            keyframe: keys[0].clone(),
        }),
        keyframe(KeyframeEdit::UpsertNumber {
            clip_id: clip_id.clone(),
            target: NumberCurveTarget::VisualOpacity,
            keyframe: keys[1].clone(),
        }),
        keyframe(KeyframeEdit::UpsertNumber {
            clip_id: clip_id.clone(),
            target: NumberCurveTarget::AudioGain,
            keyframe: keys[2].clone(),
        }),
        keyframe(KeyframeEdit::UpsertNumber {
            clip_id: clip_id.clone(),
            target: NumberCurveTarget::AudioPan,
            keyframe: keys[3].clone(),
        }),
        keyframe(KeyframeEdit::UpsertNumber {
            clip_id: clip_id.clone(),
            target: NumberCurveTarget::EffectParameter {
                effect_id: EffectId::new("fx_color").unwrap(),
                parameter: EffectParameter::Brightness,
            },
            keyframe: keys[4].clone(),
        }),
        keyframe(KeyframeEdit::UpsertPoint {
            clip_id: clip_id.clone(),
            target: PointCurveTarget::VisualPosition,
            keyframe: point_key("kf_position_edit", 0),
        }),
        keyframe(KeyframeEdit::UpsertVec2 {
            clip_id: clip_id.clone(),
            target: Vec2CurveTarget::VisualScale,
            keyframe: vec2_key("kf_scale_edit", 0, 1.1),
        }),
        keyframe(KeyframeEdit::UpsertRect {
            clip_id: clip_id.clone(),
            target: RectCurveTarget::VisualCrop,
            keyframe: Keyframe {
                id: KeyframeId::new("kf_crop_edit").unwrap(),
                time: time(0),
                value: Rect {
                    x: 0.1,
                    y: 0.1,
                    width: 0.8,
                    height: 0.8,
                },
                interpolation: Interpolation::Linear,
            },
        }),
    ];
    let outcome = apply_edit_batch(
        &project,
        &batch("op_keyframe_targets", &project, operations),
    );
    let (updated, changed) = match outcome {
        EditOutcome::Applied {
            project,
            changed_objects,
            ..
        } => (project, changed_objects),
        other => panic!("expected applied, got {other:?}"),
    };
    assert!(changed.contains(&ChangedObjectId::Effect {
        id: EffectId::new("fx_color").unwrap()
    }));
    for key in keys {
        assert!(changed.contains(&ChangedObjectId::Keyframe { id: key.id }));
    }
    assert!(changed.contains(&ChangedObjectId::Keyframe {
        id: KeyframeId::new("kf_crop_edit").unwrap()
    }));
    let clip = &updated.project.sequences[0].tracks[0].clips[0];
    assert_eq!(
        clip.visual
            .as_ref()
            .unwrap()
            .opacity
            .keyframes()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        clip.audio.as_ref().unwrap().gain.keyframes().unwrap().len(),
        1
    );
}

#[test]
fn upsert_rejects_foreign_ids_duplicate_times_missing_targets_and_locks() {
    let project = sample_project();
    let id = ItemId::new("itm_video").unwrap();
    let failures = [
        KeyframeEdit::UpsertNumber {
            clip_id: id.clone(),
            target: NumberCurveTarget::AudioGain,
            keyframe: number_key("kf_opacity_start", 10, 1.0),
        },
        KeyframeEdit::UpsertNumber {
            clip_id: id.clone(),
            target: NumberCurveTarget::VisualOpacity,
            keyframe: number_key("kf_duplicate_time", 0, 0.5),
        },
        KeyframeEdit::UpsertNumber {
            clip_id: id.clone(),
            target: NumberCurveTarget::EffectParameter {
                effect_id: EffectId::new("fx_color").unwrap(),
                parameter: EffectParameter::TargetLufs,
            },
            keyframe: number_key("kf_missing_parameter", 0, 0.5),
        },
        KeyframeEdit::UpsertRect {
            clip_id: id.clone(),
            target: RectCurveTarget::VisualCrop,
            keyframe: Keyframe {
                id: KeyframeId::new("kf_missing_crop").unwrap(),
                time: time(0),
                value: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                },
                interpolation: Interpolation::Linear,
            },
        },
    ];
    for (index, edit) in failures.into_iter().enumerate() {
        let value = batch(
            &format!("op_bad_keyframe_{index}"),
            &project,
            vec![keyframe(edit)],
        );
        assert_rejected(apply_edit_batch(&project, &value), "EDIT_REJECTED");
    }
    let mut locked = project.clone();
    locked.project.sequences[0].tracks[0].state.locked = true;
    let value = batch(
        "op_locked_keyframe",
        &locked,
        vec![keyframe(KeyframeEdit::UpsertNumber {
            clip_id: id,
            target: NumberCurveTarget::AudioGain,
            keyframe: number_key("kf_locked", 0, 1.0),
        })],
    );
    assert_rejected(apply_edit_batch(&locked, &value), "EDIT_REJECTED");
}

fn keyframe(edit: KeyframeEdit) -> EditOperation {
    EditOperation::EditKeyframe { edit }
}
