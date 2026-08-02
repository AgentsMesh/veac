use crate::support::{clips, lower_example};
use veac_ir::{
    Anchor, Animatable, ClipSource, Generator, Interpolation, Placement, ProjectEnvelope,
    Transform2D, Vec2, VectorGeometry,
};

fn transform(project: &ProjectEnvelope) -> &Transform2D {
    clips(project)
        .find(|clip| clip.id.as_str() == "itm_badge")
        .and_then(|clip| clip.visual.as_ref())
        .map(|visual| &visual.transform)
        .expect("badge visual transform")
}

#[test]
fn transform_example_lowers_all_interpolation_primitives() {
    let project = lower_example("transforms-and-animation/main.veac");
    let keyframes = transform(&project)
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
    let badge = clips(&project)
        .find(|clip| clip.id.as_str() == "itm_badge")
        .expect("transform badge");
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
