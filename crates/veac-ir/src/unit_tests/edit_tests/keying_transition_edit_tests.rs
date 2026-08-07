use super::*;

#[test]
fn every_parameterized_transition_kind_is_a_typed_atomic_edit() {
    let project = transition_ready_project(60);
    let kinds = [
        TransitionKind::Dissolve,
        TransitionKind::Fade {
            color: FadeColor::White,
        },
        TransitionKind::Wipe {
            direction: CardinalDirection::Down,
            angle_degrees: 15.0,
            softness: 0.2,
        },
        TransitionKind::Slide {
            direction: CardinalDirection::Left,
            amount: 1.5,
        },
        TransitionKind::Zoom {
            direction: ZoomDirection::Out,
            amount: 2.0,
        },
        TransitionKind::Circle {
            direction: CircleDirection::Open,
            softness: 0.1,
        },
        TransitionKind::Pixelize { amount: 0.8 },
    ];
    for (index, kind) in kinds.into_iter().enumerate() {
        let transition = Transition {
            kind,
            duration: time(60),
            alignment: TransitionAlignment::Centered,
        };
        let edit = batch(
            &format!("op_typed_transition_{index}"),
            &project,
            vec![EditOperation::SetTransition {
                clip_id: ItemId::new("itm_video").unwrap(),
                transition: Some(transition.clone()),
            }],
        );
        let updated = applied(apply_edit_batch(&project, &edit));
        assert_eq!(
            crate::test_support::transition_from(&updated, "itm_video"),
            Some(&transition)
        );
    }

    let invalid = batch(
        "op_invalid_typed_transition",
        &project,
        vec![EditOperation::SetTransition {
            clip_id: ItemId::new("itm_video").unwrap(),
            transition: Some(Transition {
                kind: TransitionKind::Zoom {
                    direction: ZoomDirection::In,
                    amount: 9.0,
                },
                duration: time(60),
                alignment: TransitionAlignment::Centered,
            }),
        }],
    );
    assert_rejected(apply_edit_batch(&project, &invalid), "TRANSITION");
}

#[test]
fn luma_key_and_spill_parameters_edit_with_registry_type_and_range_guards() {
    let mut project = sample_project();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.effects = vec![luma_effect(), spill_effect()];
    let edits = vec![
        parameter(
            "fx_luma_edit",
            EffectParameter::Threshold,
            EffectParameterValue::Curve(Animatable::constant(0.4)),
        ),
        parameter(
            "fx_spill_edit",
            EffectParameter::Amount,
            EffectParameterValue::Curve(Animatable::constant(0.9)),
        ),
    ];
    let updated = applied(apply_edit_batch(
        &project,
        &batch("op_keying_parameters", &project, edits),
    ));
    let effects = &updated.project.sequences[0].tracks[0].clips[0].effects;
    assert_eq!(
        effects[0].effect.curve(EffectParameter::Threshold),
        Some(&Animatable::constant(0.4))
    );
    assert_eq!(
        effects[1].effect.curve(EffectParameter::Amount),
        Some(&Animatable::constant(0.9))
    );

    for (index, edit) in [
        parameter(
            "fx_luma_edit",
            EffectParameter::Threshold,
            EffectParameterValue::Boolean(true),
        ),
        parameter(
            "fx_spill_edit",
            EffectParameter::Amount,
            EffectParameterValue::Curve(Animatable::constant(2.0)),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let invalid = batch(&format!("op_bad_keying_{index}"), &project, vec![edit]);
        assert_rejected(
            apply_edit_batch(&project, &invalid),
            if index == 0 {
                "EDIT_REJECTED"
            } else {
                "EFFECT_PARAMETER_RANGE"
            },
        );
    }
}

fn parameter(
    effect: &str,
    parameter: EffectParameter,
    value: EffectParameterValue,
) -> EditOperation {
    EditOperation::EditEffectParameter {
        edit: EffectParameterEdit::Set {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect_id: EffectId::new(effect).unwrap(),
            parameter,
            value,
        },
    }
}

fn luma_effect() -> EffectInstance {
    effect(
        "fx_luma_edit",
        Effect::VideoLumaKey {
            threshold: Animatable::constant(0.2),
            tolerance: Animatable::constant(0.1),
            softness: Animatable::constant(0.1),
            invert: false,
        },
    )
}

fn spill_effect() -> EffectInstance {
    effect(
        "fx_spill_edit",
        Effect::VideoChromaSpill {
            color: Color {
                red: 0,
                green: 255,
                blue: 0,
                alpha: 255,
            },
            amount: Animatable::constant(0.5),
            range: Animatable::constant(0.2),
        },
    )
}

fn effect(id: &str, effect: Effect) -> EffectInstance {
    EffectInstance {
        id: EffectId::new(id).unwrap(),
        enabled: true,
        enable_range: None,
        effect,
    }
}
