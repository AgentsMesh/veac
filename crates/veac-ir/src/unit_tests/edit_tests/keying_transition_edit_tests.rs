use std::collections::BTreeMap;

use super::*;

#[test]
fn every_parameterized_transition_kind_is_a_typed_atomic_edit() {
    let project = magnetic_project();
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
            "threshold",
            ParameterValue::Number { value: 0.4 },
        ),
        parameter(
            "fx_spill_edit",
            "amount",
            ParameterValue::Number { value: 0.9 },
        ),
    ];
    let updated = applied(apply_edit_batch(
        &project,
        &batch("op_keying_parameters", &project, edits),
    ));
    let effects = &updated.project.sequences[0].tracks[0].clips[0].effects;
    assert_eq!(
        effects[0].parameters["threshold"],
        ParameterValue::Number { value: 0.4 }
    );
    assert_eq!(
        effects[1].parameters["amount"],
        ParameterValue::Number { value: 0.9 }
    );

    for (index, edit) in [
        parameter(
            "fx_luma_edit",
            "threshold",
            ParameterValue::Text {
                value: "wrong".to_owned(),
            },
        ),
        parameter(
            "fx_spill_edit",
            "amount",
            ParameterValue::Number { value: 2.0 },
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let invalid = batch(&format!("op_bad_keying_{index}"), &project, vec![edit]);
        assert_rejected(
            apply_edit_batch(&project, &invalid),
            "EFFECT_PARAMETER_TYPE",
        );
    }
}

fn parameter(effect: &str, name: &str, value: ParameterValue) -> EditOperation {
    EditOperation::EditEffectParameter {
        edit: EffectParameterEdit::Set {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect_id: EffectId::new(effect).unwrap(),
            name: name.to_owned(),
            value,
        },
    }
}

fn luma_effect() -> EffectInstance {
    effect(
        "fx_luma_edit",
        "video.luma_key",
        BTreeMap::from([
            (
                "threshold".to_owned(),
                ParameterValue::Number { value: 0.2 },
            ),
            (
                "invert".to_owned(),
                ParameterValue::Boolean { value: false },
            ),
        ]),
    )
}

fn spill_effect() -> EffectInstance {
    effect(
        "fx_spill_edit",
        "video.chroma_spill",
        BTreeMap::from([
            (
                "color".to_owned(),
                ParameterValue::Color {
                    value: Color {
                        red: 0,
                        green: 255,
                        blue: 0,
                        alpha: 255,
                    },
                },
            ),
            ("amount".to_owned(), ParameterValue::Number { value: 0.5 }),
        ]),
    )
}

fn effect(
    id: &str,
    effect_type: &str,
    parameters: BTreeMap<String, ParameterValue>,
) -> EffectInstance {
    EffectInstance {
        id: EffectId::new(id).unwrap(),
        effect_type: effect_type.to_owned(),
        enabled: true,
        enable_range: None,
        parameters,
    }
}
