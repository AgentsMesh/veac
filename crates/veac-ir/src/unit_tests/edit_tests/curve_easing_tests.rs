use super::*;

#[test]
fn split_preserves_named_and_asymmetric_bezier_curves_at_global_times() {
    let easings = [
        Interpolation::EaseIn,
        Interpolation::EaseOut,
        Interpolation::EaseInOut,
        Interpolation::CubicBezier {
            x1: 0.08,
            y1: 0.2,
            x2: 0.82,
            y2: 0.9,
        },
    ];
    for (index, easing) in easings.into_iter().enumerate() {
        let mut project = sample_project();
        set_curve(&mut project, easing);
        let original = curve(&project).clone();
        let edit = batch(
            &format!("op_easing_split_{index}"),
            &project,
            vec![EditOperation::SplitClip {
                clip_id: ItemId::new("itm_video").unwrap(),
                at: time(210),
                right_clip_id: ItemId::new(format!("itm_easing_right_{index}")).unwrap(),
                relation_fragments: vec![],
            }],
        );
        let first = applied(apply_edit_batch(&project, &edit));
        let second = applied(apply_edit_batch(&project, &edit));
        assert_eq!(first, second);
        let clips = &first.project.sequences[0].tracks[0].clips;
        for global in [0, 45, 120, 209, 210, 275, 420, 599, 600] {
            let actual = if global <= 210 {
                sample(curve_on(&clips[0]), global)
            } else {
                sample(curve_on(&clips[1]), global - 210)
            };
            assert_close(actual, sample(&original, global));
        }
        assert_close(
            sample(curve_on(&clips[0]), 210),
            sample(curve_on(&clips[1]), 0),
        );
    }
}

#[test]
fn trim_in_preserves_the_remaining_asymmetric_bezier_curve() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].placement_mode = PlacementMode::Free;
    set_curve(
        &mut project,
        Interpolation::CubicBezier {
            x1: 0.08,
            y1: 0.2,
            x2: 0.82,
            y2: 0.9,
        },
    );
    let original = curve(&project).clone();
    let edit = batch(
        "op_easing_trim",
        &project,
        vec![EditOperation::TrimClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            edge: TrimEdge::In,
            delta: time(120),
            ripple: false,
        }],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let trimmed = curve(&result);
    for global in [120, 180, 300, 450, 599, 600] {
        assert_close(sample(trimmed, global - 120), sample(&original, global));
    }
}

#[test]
fn noncanonical_overshoot_easing_rejects_the_atomic_batch() {
    let mut project = sample_project();
    set_curve(
        &mut project,
        Interpolation::CubicBezier {
            x1: 1.0 / 3.0,
            y1: 1.0,
            x2: 2.0 / 3.0,
            y2: -4.0 / 3.0,
        },
    );
    let before = project.clone();
    let edit = batch(
        "op_unrepresentable_easing",
        &project,
        vec![EditOperation::SplitClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            at: time(300),
            right_clip_id: ItemId::new("itm_unrepresentable_right").unwrap(),
            relation_fragments: vec![],
        }],
    );
    assert_rejected(apply_edit_batch(&project, &edit), "BEZIER");
    assert_eq!(project, before);
}

fn set_curve(project: &mut ProjectEnvelope, interpolation: Interpolation) {
    project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .opacity = Animatable::Keyframes {
        keyframes: vec![
            Keyframe {
                id: KeyframeId::new("kf_easing_start").unwrap(),
                time: time(0),
                value: 0.0,
                interpolation,
            },
            Keyframe {
                id: KeyframeId::new("kf_easing_end").unwrap(),
                time: time(600),
                value: 1.0,
                interpolation: Interpolation::Linear,
            },
        ],
    };
}

fn curve(project: &ProjectEnvelope) -> &Animatable<f64> {
    curve_on(&project.project.sequences[0].tracks[0].clips[0])
}

fn curve_on(clip: &Clip) -> &Animatable<f64> {
    &clip.visual.as_ref().unwrap().opacity
}

fn sample(curve: &Animatable<f64>, at: i64) -> f64 {
    let keys = curve.keyframes().unwrap();
    if at <= keys[0].time.value {
        return keys[0].value;
    }
    let Some(pair) = keys.windows(2).find(|pair| at <= pair[1].time.value) else {
        return keys.last().unwrap().value;
    };
    let progress =
        (at - pair[0].time.value) as f64 / (pair[1].time.value - pair[0].time.value) as f64;
    pair[0].value + (pair[1].value - pair[0].value) * pair[0].interpolation.evaluate(progress)
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}
