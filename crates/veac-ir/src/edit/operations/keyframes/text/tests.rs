use crate::test_support::{sample_project, time};

use super::*;

fn key(id: &str, at: i64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value: 1.0,
        interpolation: Interpolation::Linear,
    }
}

#[test]
fn text_keyframes_fall_through_from_reveal_to_opacity() {
    let project = sample_project();
    let mut clip = project.project.sequences[0].tracks[1].clips[0].clone();
    let ClipSource::Caption { style, .. } = &mut clip.source else {
        panic!("caption fixture")
    };
    let opacity_id = KeyframeId::new("kf_opacity_first").unwrap();
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Whole,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::Keyframes {
            keyframes: vec![key("kf_opacity_first", 0), key("kf_opacity_second", 20)],
        },
        stagger: time(0),
    });
    assert!(has(&clip, &opacity_id));
    assert_eq!(
        move_keyframe(&mut clip, &opacity_id, time(5)).unwrap(),
        Some(true)
    );
    assert!(remove(&mut clip, &opacity_id).unwrap());

    let mut no_animation = project.project.sequences[0].tracks[1].clips[0].clone();
    assert!(!has(&no_animation, &opacity_id));
    assert!(!remove(&mut no_animation, &opacity_id).unwrap());
    assert_eq!(
        move_keyframe(&mut no_animation, &opacity_id, time(1)).unwrap(),
        None
    );
    let video = &mut no_animation;
    video.source = project.project.sequences[0].tracks[0].clips[0]
        .source
        .clone();
    assert!(!has(video, &opacity_id));
}
