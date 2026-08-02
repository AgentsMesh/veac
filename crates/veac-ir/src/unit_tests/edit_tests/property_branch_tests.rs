use super::*;

#[test]
fn effect_curve_set_noop_and_remove_report_curve_ownership() {
    let mut project = sample_project();
    let effect = &mut project.project.sequences[0].tracks[0].clips[0].effects[0];
    effect.parameters.insert(
        "brightness".to_owned(),
        ParameterValue::NumberCurve {
            value: curve("kf_effect_old", 0.1, 0.2),
        },
    );
    validate(&project).unwrap();
    let replacement = ParameterValue::NumberCurve {
        value: curve("kf_effect_new", 0.3, 0.4),
    };
    let set = parameter_set(replacement.clone());
    let (updated, changed) = applied_with_changes(apply_edit_batch(
        &project,
        &batch("op_effect_curve_set", &project, vec![set.clone()]),
    ));
    for id in [
        "kf_effect_old_a",
        "kf_effect_old_b",
        "kf_effect_new_a",
        "kf_effect_new_b",
    ] {
        assert!(changed.contains(&ChangedObjectId::Keyframe {
            id: KeyframeId::new(id).unwrap(),
        }));
    }
    assert!(changed.contains(&ChangedObjectId::Effect {
        id: EffectId::new("fx_color").unwrap(),
    }));

    let noop = apply_edit_batch(
        &updated,
        &batch("op_effect_curve_noop", &updated, vec![set]),
    );
    assert!(matches!(noop, EditOutcome::NoChange { .. }));
    let remove = EditOperation::EditEffectParameter {
        edit: EffectParameterEdit::Remove {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect_id: EffectId::new("fx_color").unwrap(),
            name: "brightness".to_owned(),
        },
    };
    let (_, removed) = applied_with_changes(apply_edit_batch(
        &updated,
        &batch("op_effect_curve_remove", &updated, vec![remove]),
    ));
    for id in ["kf_effect_new_a", "kf_effect_new_b"] {
        assert!(removed.contains(&ChangedObjectId::Keyframe {
            id: KeyframeId::new(id).unwrap(),
        }));
    }
}

#[test]
fn mask_property_marks_all_curves_and_missing_components_fail_precisely() {
    let project = sample_project();
    let mut mask = project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .masks[0]
        .clone();
    mask.position = vec_curve("kf_mask_property_position", 0.4, 0.6);
    mask.scale = vec_curve("kf_mask_property_scale", 0.8, 1.0);
    mask.rotation_degrees = curve("kf_mask_property_rotation", 0.0, 5.0);
    mask.feather_pixels = curve("kf_mask_property_feather", 1.0, 4.0);
    mask.expansion_pixels = curve("kf_mask_property_expansion", 0.0, 2.0);
    let operation = EditOperation::SetVisualProperty {
        clip_id: ItemId::new("itm_video").unwrap(),
        property: VisualProperty::Masks(vec![mask]),
    };
    let (_, changed) = applied_with_changes(apply_edit_batch(
        &project,
        &batch("op_mask_property_curves", &project, vec![operation]),
    ));
    for stem in ["position", "scale", "rotation", "feather", "expansion"] {
        for suffix in ["a", "b"] {
            let id = KeyframeId::new(format!("kf_mask_property_{stem}_{suffix}")).unwrap();
            assert!(changed.contains(&ChangedObjectId::Keyframe { id }));
        }
    }

    let linked = linked_project();
    let failures = [
        EditOperation::SetVisualProperty {
            clip_id: ItemId::new("itm_audio").unwrap(),
            property: VisualProperty::Opacity(Animatable::constant(0.5)),
        },
        EditOperation::EditEffectParameter {
            edit: EffectParameterEdit::Set {
                clip_id: ItemId::new("itm_video").unwrap(),
                effect_id: EffectId::new("fx_missing").unwrap(),
                name: "brightness".to_owned(),
                value: ParameterValue::Number { value: 0.3 },
            },
        },
        EditOperation::SetTextProperty {
            clip_id: ItemId::new("itm_missing").unwrap(),
            property: TextProperty::SizePixels(30.0),
        },
    ];
    for (index, operation) in failures.into_iter().enumerate() {
        let outcome = apply_edit_batch(
            &linked,
            &batch(
                &format!("op_property_missing_{index}"),
                &linked,
                vec![operation],
            ),
        );
        assert_rejected(outcome, "EDIT_REJECTED");
    }
}

fn parameter_set(value: ParameterValue) -> EditOperation {
    EditOperation::EditEffectParameter {
        edit: EffectParameterEdit::Set {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect_id: EffectId::new("fx_color").unwrap(),
            name: "brightness".to_owned(),
            value,
        },
    }
}

fn curve(prefix: &str, first: f64, second: f64) -> Animatable<f64> {
    Animatable::Keyframes {
        keyframes: vec![
            number_key(&format!("{prefix}_a"), 0, first),
            number_key(&format!("{prefix}_b"), 60, second),
        ],
    }
}

fn vec_curve(prefix: &str, first: f64, second: f64) -> Animatable<Vec2> {
    Animatable::Keyframes {
        keyframes: vec![
            vec2_key(&format!("{prefix}_a"), 0, first),
            vec2_key(&format!("{prefix}_b"), 60, second),
        ],
    }
}

fn applied_with_changes(value: EditOutcome) -> (ProjectEnvelope, Vec<ChangedObjectId>) {
    match value {
        EditOutcome::Applied {
            project,
            changed_objects,
            ..
        } => (project, changed_objects),
        other => panic!("expected applied, got {other:?}"),
    }
}
