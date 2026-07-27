use super::*;

#[test]
fn vertical_path_and_unit_transform_contracts_are_canonical() {
    let mut vertical = sample_project();
    let style = caption_style(&mut vertical);
    style.layout.writing_mode = TextWritingMode::VerticalRl;
    style.layout.orientation = TextOrientation::Mixed;
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform {
            position_offset: Animatable::constant(point(0.0, 0.0)),
            scale: Animatable::constant(Vec2 { x: 1.2, y: 0.8 }),
            rotation_degrees: Animatable::constant(15.0),
        },
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::constant(0.75),
        stagger: crate::test_support::time(10),
    });
    validate(&vertical).unwrap();

    let mut on_path = sample_project();
    caption_style(&mut on_path).path = Some(TextPath {
        points: vec![point(10.0, 30.0), point(90.0, 30.0)],
        start_offset: percent(50.0),
        reverse: true,
        alignment: TextPathAlignment::Center,
    });
    validate(&on_path).unwrap();
    let json = canonical_json(&on_path).unwrap();
    assert!(json.contains("\"alignment\":\"center\""));
    assert!(json.contains("\"reverse\":true"));
}

#[test]
fn invalid_vertical_path_and_transform_values_are_rejected() {
    let mut short = sample_project();
    caption_style(&mut short).path = Some(TextPath {
        points: vec![point(0.0, 0.0)],
        start_offset: pixels(0.0),
        reverse: false,
        alignment: TextPathAlignment::Start,
    });
    assert_code(&validation_codes(&short), "TEXT_PATH");

    let mut over_budget = sample_project();
    caption_style(&mut over_budget).path = Some(TextPath {
        points: (0..257).map(|index| point(index as f64, 0.0)).collect(),
        start_offset: pixels(0.0),
        reverse: false,
        alignment: TextPathAlignment::Start,
    });
    assert_code(&validation_codes(&over_budget), "TEXT_PATH");

    let mut repeated = sample_project();
    let style = caption_style(&mut repeated);
    style.path = Some(TextPath {
        points: vec![point(0.0, 0.0), point(0.0, 0.0)],
        start_offset: pixels(0.0),
        reverse: false,
        alignment: TextPathAlignment::Start,
    });
    style.layout.writing_mode = TextWritingMode::VerticalLr;
    assert_code(&validation_codes(&repeated), "TEXT_PATH");

    let mut bad_animation = sample_project();
    let style = caption_style(&mut bad_animation);
    style.layout.writing_mode = TextWritingMode::VerticalRl;
    style.layout.wrap = TextWrap::Word;
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Word,
        transform: TextUnitTransform {
            position_offset: Animatable::constant(point(f64::NAN, 0.0)),
            scale: Animatable::constant(Vec2 { x: 0.0, y: 1.0 }),
            rotation_degrees: Animatable::constant(f64::INFINITY),
        },
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::constant(1.0),
        stagger: crate::test_support::time(0),
    });
    let codes = validation_codes(&bad_animation);
    assert_code(&codes, "TEXT_LAYOUT");
    assert_code(&codes, "ANIMATION_VALUE");
}

fn caption_style(project: &mut ProjectEnvelope) -> &mut TextStyle {
    match &mut project.project.sequences[0].tracks[1].clips[0].source {
        ClipSource::Caption { style, .. } => style,
        _ => panic!("caption fixture"),
    }
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

fn percent(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Percent,
    }
}
