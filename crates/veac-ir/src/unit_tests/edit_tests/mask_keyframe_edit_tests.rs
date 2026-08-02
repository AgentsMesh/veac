use super::*;

#[test]
fn every_mask_curve_target_supports_upsert_move_and_remove() {
    let project = mask_project();
    let clip_id = ItemId::new("itm_video").unwrap();
    let mut seed = Vec::new();
    for (target, name, first, second) in [
        (
            NumberCurveTarget::MaskRotationDegrees { mask_index: 0 },
            "rotation",
            0.0,
            45.0,
        ),
        (
            NumberCurveTarget::MaskFeatherPixels { mask_index: 0 },
            "feather",
            1.0,
            4.0,
        ),
        (
            NumberCurveTarget::MaskExpansionPixels { mask_index: 0 },
            "expansion",
            -2.0,
            3.0,
        ),
    ] {
        for (suffix, at, value) in [("a", 0, first), ("b", 30, second)] {
            seed.push(keyframe(KeyframeEdit::UpsertNumber {
                clip_id: clip_id.clone(),
                target: target.clone(),
                keyframe: number_key(&format!("kf_mask_{name}_{suffix}"), at, value),
            }));
        }
    }
    for (target, name, first, second) in [
        (
            Vec2CurveTarget::MaskPosition { mask_index: 0 },
            "position",
            0.4,
            0.6,
        ),
        (
            Vec2CurveTarget::MaskScale { mask_index: 0 },
            "scale",
            0.8,
            1.2,
        ),
    ] {
        for (suffix, at, value) in [("a", 0, first), ("b", 30, second)] {
            seed.push(keyframe(KeyframeEdit::UpsertVec2 {
                clip_id: clip_id.clone(),
                target,
                keyframe: vec2_key(&format!("kf_mask_{name}_{suffix}"), at, value),
            }));
        }
    }
    let seeded = applied(apply_edit_batch(
        &project,
        &batch("op_seed_mask_curves", &project, seed),
    ));
    assert_mask_lengths(&seeded, 2);

    let names = ["rotation", "feather", "expansion", "position", "scale"];
    let moves = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            keyframe(KeyframeEdit::Move {
                clip_id: clip_id.clone(),
                keyframe_id: KeyframeId::new(format!("kf_mask_{name}_a")).unwrap(),
                time: time(5 + index as i64),
            })
        })
        .collect();
    let moved = applied(apply_edit_batch(
        &seeded,
        &batch("op_move_mask_curves", &seeded, moves),
    ));
    let removals = names
        .iter()
        .map(|name| {
            keyframe(KeyframeEdit::Remove {
                clip_id: clip_id.clone(),
                keyframe_id: KeyframeId::new(format!("kf_mask_{name}_a")).unwrap(),
            })
        })
        .collect();
    let removed = applied(apply_edit_batch(
        &moved,
        &batch("op_remove_mask_curves", &moved, removals),
    ));
    assert_mask_lengths(&removed, 1);
}

#[test]
fn missing_mask_curve_target_rejects_the_batch_atomically() {
    let project = mask_project();
    let invalid = batch(
        "op_missing_mask_curve",
        &project,
        vec![keyframe(KeyframeEdit::UpsertNumber {
            clip_id: ItemId::new("itm_video").unwrap(),
            target: NumberCurveTarget::MaskFeatherPixels { mask_index: 9 },
            keyframe: number_key("kf_missing_mask", 0, 2.0),
        })],
    );
    assert_rejected(apply_edit_batch(&project, &invalid), "EDIT_REJECTED");
    assert!(project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .masks[0]
        .feather_pixels
        .keyframes()
        .is_none());
}

fn mask_project() -> ProjectEnvelope {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .masks = vec![Mask {
        shape: MaskShape::Circle,
        position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
        scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
        rotation_degrees: Animatable::constant(0.0),
        feather_pixels: Animatable::constant(0.0),
        expansion_pixels: Animatable::constant(0.0),
        invert: false,
    }];
    project
}

fn assert_mask_lengths(project: &ProjectEnvelope, expected: usize) {
    let mask = &project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .masks[0];
    assert_eq!(mask.position.keyframes().unwrap().len(), expected);
    assert_eq!(mask.scale.keyframes().unwrap().len(), expected);
    assert_eq!(mask.rotation_degrees.keyframes().unwrap().len(), expected);
    assert_eq!(mask.feather_pixels.keyframes().unwrap().len(), expected);
    assert_eq!(mask.expansion_pixels.keyframes().unwrap().len(), expected);
}

fn keyframe(edit: KeyframeEdit) -> EditOperation {
    EditOperation::EditKeyframe { edit }
}
