use crate::test_support::time;

use super::*;

fn blur_effect() -> EffectInstance {
    EffectInstance {
        id: EffectId::new("fx_blur_added").unwrap(),
        enabled: true,
        enable_range: None,
        effect: Effect::VideoBlur {
            radius: Animatable::constant(3.0),
        },
    }
}

#[test]
fn effect_stack_uses_stable_anchors_for_add_move_and_remove() {
    let project = sample_project();
    let add = batch(
        "op_add_effect",
        &project,
        vec![EditOperation::AddEffect {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect: blur_effect(),
            before_id: Some(EffectId::new("fx_color").unwrap()),
            after_id: None,
        }],
    );
    let added = applied(apply_edit_batch(&project, &add));
    let effects = &added.project.sequences[0].tracks[0].clips[0].effects;
    assert_eq!(effects[0].id.as_str(), "fx_blur_added");

    let move_effect = batch(
        "op_move_effect",
        &added,
        vec![EditOperation::MoveEffect {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect_id: EffectId::new("fx_blur_added").unwrap(),
            before_id: None,
            after_id: Some(EffectId::new("fx_color").unwrap()),
        }],
    );
    let moved = applied(apply_edit_batch(&added, &move_effect));
    let effects = &moved.project.sequences[0].tracks[0].clips[0].effects;
    assert_eq!(effects[1].id.as_str(), "fx_blur_added");

    let remove = batch(
        "op_remove_effect",
        &moved,
        vec![EditOperation::RemoveEffect {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect_id: EffectId::new("fx_blur_added").unwrap(),
        }],
    );
    let removed = applied(apply_edit_batch(&moved, &remove));
    assert_eq!(
        removed.project.sequences[0].tracks[0].clips[0]
            .effects
            .len(),
        1
    );
}

#[test]
fn effect_stack_rejects_duplicate_ambiguous_self_and_missing_anchors() {
    let project = sample_project();
    let failures = [
        EditOperation::AddEffect {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect: EffectInstance {
                id: EffectId::new("fx_color").unwrap(),
                ..blur_effect()
            },
            before_id: None,
            after_id: None,
        },
        EditOperation::AddEffect {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect: blur_effect(),
            before_id: Some(EffectId::new("fx_color").unwrap()),
            after_id: Some(EffectId::new("fx_color").unwrap()),
        },
        EditOperation::RemoveEffect {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect_id: EffectId::new("fx_missing").unwrap(),
        },
        EditOperation::MoveEffect {
            clip_id: ItemId::new("itm_video").unwrap(),
            effect_id: EffectId::new("fx_color").unwrap(),
            before_id: Some(EffectId::new("fx_color").unwrap()),
            after_id: None,
        },
    ];
    for (index, operation) in failures.into_iter().enumerate() {
        let edit = batch(&format!("op_bad_stack_{index}"), &project, vec![operation]);
        assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
    }
}

#[test]
fn common_properties_and_transition_are_typed_mutations() {
    let project = sample_project();
    for (index, operation) in [
        EditOperation::SetVisual {
            clip_id: ItemId::new("itm_video").unwrap(),
            visual: None,
        },
        EditOperation::SetAudio {
            clip_id: ItemId::new("itm_video").unwrap(),
            audio: None,
        },
    ]
    .into_iter()
    .enumerate()
    {
        let edit = batch(&format!("op_common_{index}"), &project, vec![operation]);
        assert!(matches!(
            apply_edit_batch(&project, &edit),
            EditOutcome::Applied { .. }
        ));
    }

    let timeline = transition_ready_project(60);
    let transition = Transition {
        kind: TransitionKind::Dissolve,
        duration: time(60),
        alignment: TransitionAlignment::Centered,
    };
    let edit = batch(
        "op_transition",
        &timeline,
        vec![EditOperation::SetTransition {
            clip_id: ItemId::new("itm_video").unwrap(),
            transition: Some(transition.clone()),
        }],
    );
    let result = applied(apply_edit_batch(&timeline, &edit));
    assert_eq!(
        crate::test_support::transition_from(&result, "itm_video"),
        Some(&transition)
    );
}
