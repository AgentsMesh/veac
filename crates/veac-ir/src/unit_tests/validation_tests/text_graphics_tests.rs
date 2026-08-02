use crate::test_support::time;

use super::*;

#[test]
fn advanced_text_and_generated_graphics_are_canonical_valid_data() {
    let mut project = sample_project();
    let caption = &mut project.project.sequences[0].tracks[1].clips[0];
    let ClipSource::Caption { style, .. } = &mut caption.source else {
        panic!("caption fixture")
    };
    style.fallback_fonts.push(FontRef::Material {
        material_id: MaterialId::new("med_font").unwrap(),
    });
    style.font_weight = FontWeight::Bold;
    style.font_style = FontStyle::Italic;
    style.tracking_pixels = 1.5;
    style.line_height = 1.25;
    style.layout = TextLayout {
        box_width_pixels: Some(320.0),
        box_height_pixels: Some(100.0),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        horizontal_alignment: HorizontalTextAlignment::Left,
        vertical_alignment: VerticalTextAlignment::Top,
        ..TextLayout::default()
    };
    style.spans.push(TextSpan {
        start: 0,
        end: 2,
        font: None,
        font_weight: Some(FontWeight::Black),
        font_style: None,
        size_pixels: Some(52.0),
        color: None,
    });
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Word,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::Keyframes {
            keyframes: vec![
                key("kf_text_opacity_a", 0, 0.0),
                key("kf_text_opacity_b", 300, 1.0),
            ],
        },
        stagger: time(20),
    });
    validate(&project).unwrap();

    let video = &mut project.project.sequences[0].tracks[0].clips[0];
    video.source_mapping = None;
    video.audio = None;
    video.source = ClipSource::Generated {
        generator: Generator::Shape {
            shape: VectorShape {
                geometry: VectorGeometry::Polygon {
                    points: vec![point(0.1, 0.1), point(0.9, 0.1), point(0.5, 0.9)],
                },
                fill: Some(Paint::Gradient {
                    gradient: gradient(),
                }),
                stroke: Some(VectorStroke {
                    paint: Paint::Solid {
                        color: Color {
                            red: 255,
                            green: 0,
                            blue: 0,
                            alpha: 255,
                        },
                    },
                    width_pixels: 3.0,
                }),
            },
        },
    };
    validate(&project).unwrap();
}

#[test]
fn text_and_graphics_invariants_fail_with_specific_diagnostics() {
    let mut project = sample_project();
    let caption = &mut project.project.sequences[0].tracks[1].clips[0];
    let ClipSource::Caption { style, .. } = &mut caption.source else {
        panic!("caption fixture")
    };
    style.layout.wrap = TextWrap::Character;
    style.spans.push(TextSpan {
        start: 4,
        end: 99,
        font: None,
        font_weight: None,
        font_style: None,
        size_pixels: Some(-1.0),
        color: None,
    });
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(2.0),
        highlight: None,
        opacity: Animatable::constant(1.0),
        stagger: time(0),
    });
    let video = &mut project.project.sequences[0].tracks[0].clips[0];
    video.source_mapping = None;
    video.audio = None;
    video.source = ClipSource::Generated {
        generator: Generator::Gradient {
            gradient: Gradient::Linear {
                start: point(0.0, 0.0),
                end: point(0.0, 0.0),
                stops: vec![GradientStop {
                    offset: 0.5,
                    color: Color {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 255,
                    },
                }],
            },
        },
    };
    let codes = validation_codes(&project);
    for code in [
        "TEXT_LAYOUT",
        "TEXT_SPAN",
        "ANIMATION_VALUE",
        "GENERATOR_GRADIENT",
    ] {
        assert_code(&codes, code);
    }
}

fn point(x: f64, y: f64) -> Vec2 {
    Vec2 { x, y }
}

fn gradient() -> Gradient {
    Gradient::Radial {
        center: point(0.5, 0.5),
        radius: 0.7,
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color {
                    red: 0,
                    green: 0,
                    blue: 255,
                    alpha: 255,
                },
            },
            GradientStop {
                offset: 1.0,
                color: Color {
                    red: 0,
                    green: 255,
                    blue: 0,
                    alpha: 255,
                },
            },
        ],
    }
}

fn key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Linear,
    }
}
