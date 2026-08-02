use super::{mechanism_helpers::text_clip, support::*};
use crate::{canonical::*, resolve, ResolvedClipSource};

#[test]
fn writing_path_and_unit_transform_are_owned_plan_data() {
    let mut vertical = project();
    vertical.project.materials.push(font_material("med_font"));
    let mut clip = text_clip(false);
    let ClipSource::Text { style, .. } = &mut clip.source else {
        panic!("text")
    };
    style.layout.writing_mode = TextWritingMode::VerticalLr;
    style.layout.orientation = TextOrientation::Upright;
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Word,
        transform: TextUnitTransform {
            position_offset: Animatable::constant(point(4.0, 8.0)),
            scale: Animatable::constant(Vec2 { x: 1.1, y: 0.9 }),
            rotation_degrees: Animatable::constant(12.0),
        },
        reveal: Animatable::constant(1.0),
        opacity: Animatable::constant(1.0),
        stagger: time(10),
        highlight: None,
    });
    vertical.project.sequences[0].tracks.push(track(
        "trk_vertical",
        TrackKind::Visual,
        10,
        vec![clip],
    ));
    let plan = resolve(&vertical, None).unwrap().remove(0);
    let ResolvedClipSource::Text { content } = &plan.sequences[0].tracks[1].clips[0].source else {
        panic!("resolved text")
    };
    let style = content.styled().unwrap();
    assert_eq!(style.layout.writing_mode, TextWritingMode::VerticalLr);
    assert_eq!(
        style.animation.as_ref().unwrap().transform.scale,
        Animatable::constant(Vec2 { x: 1.1, y: 0.9 })
    );

    let mut curved = vertical;
    let ClipSource::Text { style, .. } = &mut curved.project.sequences[0].tracks[1].clips[0].source
    else {
        panic!("text")
    };
    style.layout.writing_mode = TextWritingMode::HorizontalTb;
    style.path = Some(TextPath {
        points: vec![point(0.0, 30.0), point(300.0, 30.0)],
        start_offset: pixels(150.0),
        reverse: false,
        alignment: TextPathAlignment::Center,
    });
    let plan = resolve(&curved, None).unwrap().remove(0);
    let ResolvedClipSource::Text { content } = &plan.sequences[0].tracks[1].clips[0].source else {
        panic!("resolved text")
    };
    assert_eq!(
        content
            .styled()
            .unwrap()
            .path
            .as_ref()
            .unwrap()
            .points
            .len(),
        2
    );
    assert_eq!(
        serde_json::from_str::<crate::ResolvedRenderPlan>(
            &crate::canonical_plan_json(&plan).unwrap()
        )
        .unwrap(),
        plan
    );
}

fn point(x: f64, y: f64) -> Point {
    Point {
        x: pixels(x),
        y: pixels(y),
    }
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}
