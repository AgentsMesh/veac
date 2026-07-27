use super::*;

#[test]
fn move_and_remove_find_keyframes_across_visual_audio_and_effect_curves() {
    let seeded = seeded_project();
    let id = ItemId::new("itm_video").unwrap();
    let move_ids = [
        "kf_position_a",
        "kf_scale_a",
        "kf_rotation_a",
        "kf_opacity_start",
        "kf_gain_a",
        "kf_pan_a",
        "kf_effect_a",
    ];
    let moves = move_ids
        .iter()
        .enumerate()
        .map(|(index, value)| {
            keyframe(KeyframeEdit::Move {
                clip_id: id.clone(),
                keyframe_id: KeyframeId::new(*value).unwrap(),
                time: time(5 + index as i64),
            })
        })
        .collect();
    let moved = applied(apply_edit_batch(
        &seeded,
        &batch("op_move_all_keyframes", &seeded, moves),
    ));
    let remove_ids = [
        "kf_position_a",
        "kf_scale_a",
        "kf_rotation_a",
        "kf_opacity_edit",
        "kf_gain_a",
        "kf_pan_a",
        "kf_effect_a",
    ];
    let removals = remove_ids
        .iter()
        .map(|value| {
            keyframe(KeyframeEdit::Remove {
                clip_id: id.clone(),
                keyframe_id: KeyframeId::new(*value).unwrap(),
            })
        })
        .collect();
    let removed = applied(apply_edit_batch(
        &moved,
        &batch("op_remove_all_keyframes", &moved, removals),
    ));
    let clip = &removed.project.sequences[0].tracks[0].clips[0];
    assert_eq!(
        clip.audio.as_ref().unwrap().gain.keyframes().unwrap().len(),
        1
    );
    let effect_curve = match &clip.effects[0].parameters["brightness"] {
        ParameterValue::NumberCurve { value } => value,
        other => panic!("expected curve, got {other:?}"),
    };
    assert_eq!(effect_curve.keyframes().unwrap().len(), 1);
}

#[test]
fn keyframe_remove_move_and_identical_upsert_have_explicit_failure_or_noop_semantics() {
    let seeded = seeded_project();
    let id = ItemId::new("itm_video").unwrap();
    let failures = [
        KeyframeEdit::Remove {
            clip_id: id.clone(),
            keyframe_id: KeyframeId::new("kf_missing").unwrap(),
        },
        KeyframeEdit::Move {
            clip_id: id.clone(),
            keyframe_id: KeyframeId::new("kf_missing").unwrap(),
            time: time(10),
        },
        KeyframeEdit::Move {
            clip_id: id.clone(),
            keyframe_id: KeyframeId::new("kf_opacity_start").unwrap(),
            time: time(60),
        },
    ];
    for (index, edit) in failures.into_iter().enumerate() {
        let value = batch(
            &format!("op_bad_keyframe_mutation_{index}"),
            &seeded,
            vec![keyframe(edit)],
        );
        assert_rejected(apply_edit_batch(&seeded, &value), "EDIT_REJECTED");
    }
    let single = applied(apply_edit_batch(
        &sample_project(),
        &batch(
            "op_make_single_key",
            &sample_project(),
            vec![keyframe(KeyframeEdit::UpsertNumber {
                clip_id: id.clone(),
                target: NumberCurveTarget::AudioGain,
                keyframe: number_key("kf_single", 0, 1.0),
            })],
        ),
    ));
    let remove_last = batch(
        "op_remove_last_key",
        &single,
        vec![keyframe(KeyframeEdit::Remove {
            clip_id: id,
            keyframe_id: KeyframeId::new("kf_single").unwrap(),
        })],
    );
    assert_rejected(apply_edit_batch(&single, &remove_last), "EDIT_REJECTED");
}

fn seeded_project() -> ProjectEnvelope {
    let project = sample_project();
    let id = ItemId::new("itm_video").unwrap();
    let mut operations = Vec::new();
    for (target, first, second) in [
        (
            NumberCurveTarget::VisualRotationDegrees,
            "kf_rotation_a",
            "kf_rotation_b",
        ),
        (NumberCurveTarget::AudioGain, "kf_gain_a", "kf_gain_b"),
        (NumberCurveTarget::AudioPan, "kf_pan_a", "kf_pan_b"),
    ] {
        operations.push(keyframe(KeyframeEdit::UpsertNumber {
            clip_id: id.clone(),
            target: target.clone(),
            keyframe: number_key(first, 0, 0.5),
        }));
        operations.push(keyframe(KeyframeEdit::UpsertNumber {
            clip_id: id.clone(),
            target,
            keyframe: number_key(second, 20, 0.7),
        }));
    }
    operations.extend([
        keyframe(KeyframeEdit::UpsertPoint {
            clip_id: id.clone(),
            target: PointCurveTarget::VisualPosition,
            keyframe: point_key("kf_position_a", 0),
        }),
        keyframe(KeyframeEdit::UpsertPoint {
            clip_id: id.clone(),
            target: PointCurveTarget::VisualPosition,
            keyframe: point_key("kf_position_b", 20),
        }),
        keyframe(KeyframeEdit::UpsertVec2 {
            clip_id: id.clone(),
            target: Vec2CurveTarget::VisualScale,
            keyframe: vec2_key("kf_scale_a", 0, 1.0),
        }),
        keyframe(KeyframeEdit::UpsertVec2 {
            clip_id: id.clone(),
            target: Vec2CurveTarget::VisualScale,
            keyframe: vec2_key("kf_scale_b", 20, 1.2),
        }),
        keyframe(KeyframeEdit::UpsertNumber {
            clip_id: id.clone(),
            target: NumberCurveTarget::VisualOpacity,
            keyframe: number_key("kf_opacity_edit", 30, 0.5),
        }),
    ]);
    for value in ["kf_effect_a", "kf_effect_b"] {
        operations.push(keyframe(KeyframeEdit::UpsertNumber {
            clip_id: id.clone(),
            target: NumberCurveTarget::EffectParameter {
                effect_id: EffectId::new("fx_color").unwrap(),
                name: "brightness".to_owned(),
            },
            keyframe: number_key(value, if value.ends_with('a') { 0 } else { 20 }, 0.2),
        }));
    }
    applied(apply_edit_batch(
        &project,
        &batch("op_seed_keyframes", &project, operations),
    ))
}

fn keyframe(edit: KeyframeEdit) -> EditOperation {
    EditOperation::EditKeyframe { edit }
}
