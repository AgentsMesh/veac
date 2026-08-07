use super::*;
use crate::test_support::{identity_layout_visual, sample_project};

#[test]
fn animated_scale_uses_each_axis_maximum() {
    let value = Animatable::Keyframes {
        keyframes: vec![
            Keyframe {
                id: KeyframeId::new("kf_budget_a").unwrap(),
                time: RationalTime::new(0, 600).unwrap(),
                value: Vec2 { x: 2.0, y: 3.0 },
                interpolation: Interpolation::Linear,
            },
            Keyframe {
                id: KeyframeId::new("kf_budget_b").unwrap(),
                time: RationalTime::new(1, 600).unwrap(),
                value: Vec2 { x: 4.0, y: 1.0 },
                interpolation: Interpolation::Linear,
            },
        ],
    };
    assert_eq!(max_visual_scale(&value), Some((4.0, 3.0)));
    assert_eq!(
        max_visual_scale(&Animatable::constant(Vec2 { x: 0.0, y: 1.0 })),
        None
    );
}

#[test]
fn animated_scale_includes_spring_overshoot() {
    let mut first = scale_key("kf_spring_a", 0, 10.0);
    first.interpolation = Interpolation::Spring {
        frequency: 1.5,
        decay: 6.0,
        initial_velocity: 0.0,
    };
    let curve = Animatable::Keyframes {
        keyframes: vec![first, scale_key("kf_spring_b", 600, 11.0)],
    };
    let (x, y) = max_visual_scale(&curve).unwrap();
    assert!(x > 11.0);
    assert!(y > 11.0);
}

#[test]
fn generated_identity_keeps_placement_and_transform_anchor_contracts() {
    let project = sample_project();
    let mut clip = project.project.sequences[0].tracks[0].clips[0].clone();
    clip.source = ClipSource::Generated {
        generator: Generator::Transparent,
    };
    clip.effects.clear();
    let mut visual = identity_layout_visual();
    assert!(!clip_requires_canvas_placement(&clip, &visual));

    visual.placement = Placement::Anchor {
        anchor: Anchor::TopLeft,
        inset: Vec2 { x: 0.0, y: 0.0 },
    };
    assert!(clip_requires_canvas_placement(&clip, &visual));

    visual.placement = Placement::Anchor {
        anchor: Anchor::Center,
        inset: Vec2 { x: 0.0, y: 0.0 },
    };
    visual.transform.anchor = Vec2 { x: 0.0, y: 0.0 };
    assert!(clip_requires_canvas_placement(&clip, &visual));
}

#[test]
fn caption_visible_overflow_has_the_same_budget_extent_as_text() {
    let project = sample_project();
    let settings = &project.project.sequences[0].settings;
    let mut clip = project.project.sequences[0].tracks[0].clips[0].clone();
    let mut style = TextStyle::default();
    style.layout.box_width_pixels = Some(120.0);
    style.layout.box_height_pixels = Some(60.0);
    style.layout.overflow = TextOverflow::Visible;
    let mut visual = identity_layout_visual();
    visual.frame = Some(Frame {
        width: Length {
            value: 240.0,
            unit: LengthUnit::Pixels,
        },
        height: Length {
            value: 60.0,
            unit: LengthUnit::Pixels,
        },
        fit: FitMode::Contain,
    });
    clip.source = ClipSource::Text {
        text: "文本".to_owned(),
        style: style.clone(),
    };
    let text = base_extent(&clip, &visual, settings);
    clip.source = ClipSource::Caption {
        text: "字幕".to_owned(),
        speaker: None,
        cue: Box::default(),
        style,
    };
    assert_eq!(base_extent(&clip, &visual, settings), text);
}

fn scale_key(id: &str, time: i64, value: f64) -> Keyframe<Vec2> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: RationalTime::new(time, 600).unwrap(),
        value: Vec2 { x: value, y: value },
        interpolation: Interpolation::Linear,
    }
}
