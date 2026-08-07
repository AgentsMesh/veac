use crate::support::{clip_by_key, clips, lower_example};
use veac_ir::{
    evaluate_temporal_program, Anchor, Animatable, Clip, ClipSource, Generator, Interpolation,
    Placement, ProjectEnvelope, TemporalClock, TemporalEvaluationInput, TemporalEvaluationLimits,
    TemporalNodeKind, TemporalValue, Vec2, VectorGeometry,
};

fn animated_clip(project: &ProjectEnvelope) -> &Clip {
    clips(project)
        .find(|clip| {
            clip.visual
                .as_ref()
                .and_then(|visual| visual.transform.position.keyframes())
                .is_some_and(has_all_interpolations)
        })
        .expect("visual item with every interpolation primitive")
}

fn has_all_interpolations(keyframes: &[veac_ir::Keyframe<veac_ir::Point>]) -> bool {
    let has = |expected: fn(&Interpolation) -> bool| {
        keyframes
            .iter()
            .any(|keyframe| expected(&keyframe.interpolation))
    };
    has(|value| matches!(value, Interpolation::Hold))
        && has(|value| matches!(value, Interpolation::Linear))
        && has(|value| matches!(value, Interpolation::EaseIn))
        && has(|value| matches!(value, Interpolation::EaseOut))
        && has(|value| matches!(value, Interpolation::EaseInOut))
        && has(|value| matches!(value, Interpolation::CubicBezier { .. }))
        && has(|value| matches!(value, Interpolation::Spring { .. }))
}

#[test]
fn transform_example_lowers_all_interpolation_primitives() {
    let project = lower_example("transforms-and-animation/main.veac");
    let keyframes = animated_clip(&project)
        .visual
        .as_ref()
        .unwrap()
        .transform
        .position
        .keyframes()
        .expect("animated badge position");

    let has = |expected: fn(&Interpolation) -> bool| {
        keyframes
            .iter()
            .any(|keyframe| expected(&keyframe.interpolation))
    };
    assert!(has(|value| matches!(value, Interpolation::Hold)));
    assert!(has(|value| matches!(value, Interpolation::Linear)));
    assert!(has(|value| matches!(value, Interpolation::EaseIn)));
    assert!(has(|value| matches!(value, Interpolation::EaseOut)));
    assert!(has(|value| matches!(value, Interpolation::EaseInOut)));
    assert!(has(|value| matches!(
        value,
        Interpolation::CubicBezier { .. }
    )));
    assert!(has(|value| matches!(value, Interpolation::Spring { .. })));

    let spring = keyframes
        .iter()
        .find_map(|keyframe| match keyframe.interpolation {
            Interpolation::Spring {
                frequency,
                decay,
                initial_velocity,
            } => Some((frequency, decay, initial_velocity)),
            _ => None,
        });
    assert_eq!(spring, Some((1.5, 6.0, 0.0)));
}

#[test]
fn transform_example_uses_non_uniform_scale() {
    let project = lower_example("transforms-and-animation/main.veac");
    let badge = animated_clip(&project);
    let visual = badge.visual.as_ref().expect("transform badge visual");
    let transform = &visual.transform;
    let Animatable::Constant { value } = &transform.scale else {
        panic!("badge scale should be constant");
    };

    assert_eq!((value.x, value.y), (1.12, 0.92));
    assert_eq!(transform.shear, Vec2 { x: 0.22, y: -0.08 });
    assert!(transform.flip_horizontal);
    assert!(!transform.flip_vertical);
    let Animatable::Constant { value: crop } = transform.crop.as_ref().unwrap() else {
        panic!("transform crop should be constant");
    };
    assert_eq!(
        (crop.x, crop.y, crop.width, crop.height),
        (0.14, 0.04, 0.84, 0.92)
    );
    assert_eq!(transform.anchor, Vec2 { x: 1.0, y: 1.0 });
    assert_eq!(
        visual.placement,
        Placement::Anchor {
            anchor: Anchor::BottomRight,
            inset: Vec2 { x: 80.0, y: 80.0 },
        }
    );
    let ClipSource::Generated {
        generator: Generator::Shape { shape },
    } = &badge.source
    else {
        panic!("transform badge must use an asymmetric generated shape");
    };
    let VectorGeometry::Polygon { points } = &shape.geometry else {
        panic!("transform badge must remain a directional polygon");
    };
    assert_eq!(points.len(), 6);
    assert_eq!(points[2], Vec2 { x: 0.96, y: 0.5 });
    assert_eq!(points[5], Vec2 { x: 0.2, y: 0.5 });
    assert!(shape.stroke.is_some());
}

#[test]
fn transform_opacity_is_driven_by_the_authored_progress_program() {
    let project = lower_example("transforms-and-animation/main.veac");
    let badge = clip_by_key(&project, "badge");
    let Animatable::Binding { binding_id } = &badge.visual.as_ref().unwrap().opacity else {
        panic!("badge opacity must retain the authored temporal binding");
    };
    let binding = project
        .temporal
        .bindings
        .iter()
        .find(|value| &value.id == binding_id)
        .expect("authored opacity binding");
    assert_eq!(binding.clocks.len(), 1);
    assert_eq!(binding.clocks[0].clock, TemporalClock::Progress);
    let program = project
        .temporal
        .programs
        .iter()
        .find(|value| value.id == binding.program_id)
        .expect("authored opacity program");
    assert!(program.nodes.iter().any(
        |node| matches!(node.kind, TemporalNodeKind::CurveSample { ref keys, .. }
            if keys.len() == 4)
    ));
    for (progress, expected) in [(0.025, 0.5), (0.5, 1.0), (0.975, 0.5)] {
        let value = evaluate_temporal_program(
            program,
            &[TemporalEvaluationInput {
                input_id: binding.clocks[0].input_id,
                value: TemporalValue::Scalar { value: progress },
            }],
            TemporalEvaluationLimits::default(),
        )
        .expect("authored opacity evaluates");
        let TemporalValue::Scalar { value } = value else {
            panic!("authored opacity must evaluate to a scalar");
        };
        assert!((value - expected).abs() < 1e-12, "{value} != {expected}");
    }
}
