use crate::test_support::time;

use super::*;

#[test]
fn typed_visual_audio_and_effect_parameter_edits_update_only_their_targets() {
    let project = sample_project();
    let id = ItemId::new("itm_video").unwrap();
    let point = Point {
        x: Length {
            value: 10.0,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 20.0,
            unit: LengthUnit::Pixels,
        },
    };
    let operations = vec![
        visual(
            &id,
            VisualProperty::Placement(Placement::Absolute { position: point }),
        ),
        visual(&id, VisualProperty::Frame(None)),
        visual(&id, VisualProperty::Position(Animatable::constant(point))),
        visual(
            &id,
            VisualProperty::Scale(Animatable::constant(Vec2 { x: 0.2, y: 0.2 })),
        ),
        visual(&id, VisualProperty::Shear(Vec2 { x: 0.2, y: -0.1 })),
        visual(&id, VisualProperty::FlipHorizontal(true)),
        visual(&id, VisualProperty::FlipVertical(true)),
        visual(
            &id,
            VisualProperty::RotationDegrees(Animatable::constant(15.0)),
        ),
        visual(&id, VisualProperty::Anchor(Vec2 { x: 0.0, y: 1.0 })),
        visual(
            &id,
            VisualProperty::Crop(Some(Animatable::constant(Rect {
                x: 0.1,
                y: 0.1,
                width: 0.8,
                height: 0.8,
            }))),
        ),
        visual(&id, VisualProperty::Opacity(Animatable::constant(0.7))),
        visual(
            &id,
            VisualProperty::Compositing(Compositing {
                z_index: 4,
                blend_mode: BlendMode::Multiply,
            }),
        ),
        visual(&id, VisualProperty::Masks(vec![])),
        visual(&id, VisualProperty::Card(None)),
        audio(&id, AudioProperty::Gain(Animatable::constant(0.8))),
        audio(&id, AudioProperty::Pan(Animatable::constant(0.25))),
        audio(&id, AudioProperty::Muted(true)),
        audio(&id, AudioProperty::Normalize(true)),
        audio(&id, AudioProperty::PitchPolicy(PitchPolicy::FollowSpeed)),
        EditOperation::EditEffectParameter {
            edit: EffectParameterEdit::Set {
                clip_id: id.clone(),
                effect_id: EffectId::new("fx_color").unwrap(),
                name: "contrast".to_owned(),
                value: ParameterValue::Number { value: 1.5 },
            },
        },
    ];
    let updated = applied(apply_edit_batch(
        &project,
        &batch("op_typed_properties", &project, operations),
    ));
    let clip = &updated.project.sequences[0].tracks[0].clips[0];
    assert!(matches!(
        clip.visual.as_ref().unwrap().transform.crop,
        Some(Animatable::Constant { value }) if value.x == 0.1
    ));
    assert!(clip.visual.as_ref().unwrap().transform.flip_horizontal);
    assert_eq!(
        clip.visual.as_ref().unwrap().transform.shear,
        Vec2 { x: 0.2, y: -0.1 }
    );
    assert_eq!(
        clip.visual.as_ref().unwrap().transform.scale,
        Animatable::constant(Vec2 { x: 0.2, y: 0.2 })
    );
    assert!(clip.visual.as_ref().unwrap().transform.flip_vertical);
    assert!(clip.audio.as_ref().unwrap().muted);
    assert!(clip.effects[0].parameters.contains_key("contrast"));

    let remove = batch(
        "op_remove_parameter",
        &updated,
        vec![EditOperation::EditEffectParameter {
            edit: EffectParameterEdit::Remove {
                clip_id: id,
                effect_id: EffectId::new("fx_color").unwrap(),
                name: "contrast".to_owned(),
            },
        }],
    );
    let removed = applied(apply_edit_batch(&updated, &remove));
    assert!(!removed.project.sequences[0].tracks[0].clips[0].effects[0]
        .parameters
        .contains_key("contrast"));
}

#[test]
fn typed_property_edits_reject_missing_components_locks_and_invalid_values_atomically() {
    let project = sample_project();
    let failures = [
        audio(
            &ItemId::new("itm_caption").unwrap(),
            AudioProperty::Muted(true),
        ),
        EditOperation::EditEffectParameter {
            edit: EffectParameterEdit::Remove {
                clip_id: ItemId::new("itm_video").unwrap(),
                effect_id: EffectId::new("fx_color").unwrap(),
                name: "missing".to_owned(),
            },
        },
        visual(
            &ItemId::new("itm_video").unwrap(),
            VisualProperty::Opacity(Animatable::constant(2.0)),
        ),
    ];
    for (index, operation) in failures.into_iter().enumerate() {
        let edit = batch(
            &format!("op_bad_property_{index}"),
            &project,
            vec![operation],
        );
        assert_rejected(
            apply_edit_batch(&project, &edit),
            if index == 2 {
                "ANIMATION_VALUE"
            } else {
                "EDIT_REJECTED"
            },
        );
    }
    let mut locked = project.clone();
    locked.project.sequences[0].tracks[0].state.locked = true;
    let edit = batch(
        "op_locked_property",
        &locked,
        vec![audio(
            &ItemId::new("itm_video").unwrap(),
            AudioProperty::Pan(Animatable::constant(0.5)),
        )],
    );
    assert_rejected(apply_edit_batch(&locked, &edit), "EDIT_REJECTED");
}

fn visual(id: &ItemId, property: VisualProperty) -> EditOperation {
    EditOperation::SetVisualProperty {
        clip_id: id.clone(),
        property,
    }
}

fn audio(id: &ItemId, property: AudioProperty) -> EditOperation {
    EditOperation::SetAudioProperty {
        clip_id: id.clone(),
        property,
    }
}

#[test]
fn property_protocol_round_trips_curve_payloads() {
    let operation = audio(
        &ItemId::new("itm_video").unwrap(),
        AudioProperty::Gain(Animatable::Keyframes {
            keyframes: vec![Keyframe {
                id: KeyframeId::new("kf_gain").unwrap(),
                time: time(0),
                value: 1.0,
                interpolation: Interpolation::Linear,
            }],
        }),
    );
    let json = serde_json::to_string(&operation).unwrap();
    assert_eq!(
        serde_json::from_str::<EditOperation>(&json).unwrap(),
        operation
    );
}
