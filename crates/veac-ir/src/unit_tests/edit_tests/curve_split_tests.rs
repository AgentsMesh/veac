use std::collections::BTreeSet;

use super::*;

#[test]
fn split_resamples_all_curves_and_crops_effect_ranges_without_child_id_collisions() {
    let mut project = sample_project();
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    let visual = clip.visual.as_mut().unwrap();
    visual.transform.position = points();
    visual.transform.scale = vecs("kf_scale_start", "kf_scale_end", 1.0, 2.0);
    visual.transform.rotation_degrees = numbers("kf_rot_start", "kf_rot_end", 0.0, 60.0);
    visual.opacity = numbers("kf_alpha_start", "kf_alpha_end", 0.0, 1.0);
    let audio = clip.audio.as_mut().unwrap();
    audio.gain = numbers("kf_gain_start", "kf_gain_end", 0.0, 2.0);
    audio.pan = numbers("kf_pan_start", "kf_pan_end", -1.0, 1.0);
    clip.effects = vec![
        effect(
            "fx_span",
            range(100, 300),
            Some(numbers("kf_effect_start", "kf_effect_end", 0.0, 1.0)),
        ),
        effect("fx_left_only", range(0, 100), None),
        effect("fx_right_only", range(400, 100), None),
    ];
    let edit = batch(
        "op_split_curves",
        &project,
        vec![EditOperation::SplitClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            at: time(300),
            right_clip_id: ItemId::new("itm_curve_right").unwrap(),
            relation_fragments: vec![],
        }],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let left = &result.project.sequences[0].tracks[0].clips[0];
    let right = &result.project.sequences[0].tracks[0].clips[1];
    assert_curve_boundary(&left.visual.as_ref().unwrap().opacity, 0.0, 0.5, false);
    assert_curve_boundary(&right.visual.as_ref().unwrap().opacity, 1.0, 0.5, true);
    assert_curve_boundary(&left.audio.as_ref().unwrap().gain, 0.0, 1.0, false);
    assert_curve_boundary(&right.audio.as_ref().unwrap().gain, 2.0, 1.0, true);
    let left_position = left
        .visual
        .as_ref()
        .unwrap()
        .transform
        .position
        .keyframes()
        .unwrap()
        .last()
        .unwrap();
    let right_position = right
        .visual
        .as_ref()
        .unwrap()
        .transform
        .position
        .keyframes()
        .unwrap()
        .first()
        .unwrap();
    assert_eq!(left_position.value.x.value, 300.0);
    assert_eq!(right_position.value, left_position.value);
    assert_eq!(right_position.time, time(0));
    assert_eq!(left.effects.len(), 2);
    assert_eq!(right.effects.len(), 2);
    let left_span = left
        .effects
        .iter()
        .find(|value| value.id.as_str() == "fx_span")
        .unwrap();
    assert_eq!(left_span.enable_range, Some(range(100, 200)));
    let right_span = right
        .effects
        .iter()
        .find(|value| value.kind() == EffectKind::VideoColorAdjust)
        .unwrap();
    assert_ne!(right_span.id, left_span.id);
    assert_eq!(right_span.enable_range, Some(range(0, 100)));
    let left_ids = child_ids(left);
    let right_ids = child_ids(right);
    assert!(left_ids.is_disjoint(&right_ids));
}

fn points() -> Animatable<Point> {
    Animatable::Keyframes {
        keyframes: vec![
            point_key("kf_position_start", 0),
            point_key("kf_position_end", 600),
        ],
    }
}

fn vecs(start: &str, end: &str, first: f64, second: f64) -> Animatable<Vec2> {
    Animatable::Keyframes {
        keyframes: vec![vec2_key(start, 0, first), vec2_key(end, 600, second)],
    }
}

fn numbers(start: &str, end: &str, first: f64, second: f64) -> Animatable<f64> {
    Animatable::Keyframes {
        keyframes: vec![number_key(start, 0, first), number_key(end, 600, second)],
    }
}

fn effect(id: &str, enable_range: TimeRange, curve: Option<Animatable<f64>>) -> EffectInstance {
    EffectInstance {
        id: EffectId::new(id).unwrap(),
        enabled: true,
        enable_range: Some(enable_range),
        effect: match curve {
            Some(brightness) => Effect::VideoColorAdjust {
                brightness,
                contrast: Animatable::constant(1.0),
                saturation: Animatable::constant(1.0),
            },
            None => Effect::VideoBlur {
                radius: Animatable::constant(2.0),
            },
        },
    }
}

fn assert_curve_boundary(curve: &Animatable<f64>, expected_end: f64, boundary: f64, right: bool) {
    let keys = curve.keyframes().unwrap();
    let boundary_key = if right {
        &keys[0]
    } else {
        keys.last().unwrap()
    };
    assert_eq!(boundary_key.value, boundary);
    assert_eq!(boundary_key.time, if right { time(0) } else { time(300) });
    let edge = if right {
        keys.last().unwrap()
    } else {
        &keys[0]
    };
    assert_eq!(edge.value, expected_end);
}

fn child_ids(clip: &Clip) -> BTreeSet<String> {
    let mut ids: BTreeSet<_> = clip
        .effects
        .iter()
        .map(|value| value.id.to_string())
        .collect();
    let visual = clip.visual.as_ref().unwrap();
    for id in visual
        .transform
        .position
        .keyframes()
        .unwrap()
        .iter()
        .map(|value| value.id.to_string())
    {
        ids.insert(id);
    }
    for effect in &clip.effects {
        for parameter in EffectParameter::ALL {
            if let Some(value) = effect.effect.curve(parameter) {
                if value.keyframes().is_none() {
                    continue;
                }
                ids.extend(
                    value
                        .keyframes()
                        .unwrap()
                        .iter()
                        .map(|key| key.id.to_string()),
                );
            }
        }
    }
    ids
}
