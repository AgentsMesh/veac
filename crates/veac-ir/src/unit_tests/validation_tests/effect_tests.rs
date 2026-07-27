use crate::test_support::{add_transition, range, time};

use super::*;

#[test]
fn keyframes_effect_registry_and_transitions_fail_closed() {
    let mut project = sample_project();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    let visual = clip.visual.as_mut().unwrap();
    visual.opacity = Animatable::Keyframes {
        keyframes: vec![
            Keyframe {
                id: KeyframeId::new("kf_duplicate").unwrap(),
                time: time(700),
                value: 2.0,
                interpolation: Interpolation::CubicBezier {
                    x1: 0.5,
                    y1: 0.5,
                    x2: 2.0,
                    y2: f64::NAN,
                },
            },
            Keyframe {
                id: KeyframeId::new("kf_duplicate").unwrap(),
                time: time(600),
                value: 1.0,
                interpolation: Interpolation::EaseIn,
            },
        ],
    };
    visual.transform.scale = Animatable::Keyframes { keyframes: vec![] };
    let effect = &mut clip.effects[0];
    effect.id = serde_json::from_str("\"bad\"").unwrap();
    effect.effect_type = "video.color_adjsut".to_owned();
    effect.enable_range = Some(range(500, 200));
    effect.parameters.insert(
        "brighness".to_owned(),
        ParameterValue::Boolean { value: true },
    );
    let duplicate = effect.clone();
    clip.effects.push(duplicate);
    let transition = Transition {
        kind: TransitionKind::Wipe {
            direction: CardinalDirection::Left,
            angle_degrees: f64::NAN,
            softness: 2.0,
        },
        duration: time(700),
        alignment: TransitionAlignment::AfterCut,
    };
    let mut next = clip.clone();
    next.id = ItemId::new("itm_transition_target").unwrap();
    next.record_range = range(600, 600);
    project.project.sequences[0].tracks[0].clips.push(next);
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_transition_target",
        transition,
    );
    let codes = validation_codes(&project);
    for code in [
        "DUPLICATE_KEYFRAME_ID",
        "KEYFRAME_ORDER",
        "ANIMATION_VALUE",
        "BEZIER",
        "EMPTY_KEYFRAMES",
        "INVALID_ID",
        "DUPLICATE_EFFECT_ID",
        "UNKNOWN_EFFECT",
        "EFFECT_RANGE",
        "TRANSITION",
    ] {
        assert_code(&codes, code);
    }
}

#[test]
fn known_effect_rejects_unknown_parameter_type_and_range() {
    let mut project = sample_project();
    let effect = &mut project.project.sequences[0].tracks[0].clips[0].effects[0];
    effect
        .parameters
        .insert("unknown".to_owned(), ParameterValue::Number { value: 0.0 });
    effect.parameters.insert(
        "contrast".to_owned(),
        ParameterValue::Text {
            value: "bad".into(),
        },
    );
    effect.parameters.insert(
        "saturation".to_owned(),
        ParameterValue::Number { value: 9.0 },
    );
    let codes = validation_codes(&project);
    assert_code(&codes, "UNKNOWN_EFFECT_PARAMETER");
    assert_code(&codes, "EFFECT_PARAMETER_TYPE");
}

#[test]
fn transition_requires_an_adjacent_clip_with_handles() {
    let mut project = sample_project();
    let track = &mut project.project.sequences[0].tracks[0];
    track.clips[0].visual = None;
    let mut next = track.clips[0].clone();
    next.id = ItemId::new("itm_video_next").unwrap();
    next.record_range = range(600, 300);
    next.visual = None;
    next.audio = None;
    next.effects.clear();
    let transition = Transition {
        kind: TransitionKind::Dissolve,
        duration: time(60),
        alignment: TransitionAlignment::Centered,
    };
    track.clips.push(next);
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_video_next",
        transition,
    );
    validate(&project).unwrap();

    project.project.sequences[0].tracks[0].clips[1]
        .record_range
        .start = time(601);
    assert_code(&validation_codes(&project), "TRANSITION_HANDLES");
}
