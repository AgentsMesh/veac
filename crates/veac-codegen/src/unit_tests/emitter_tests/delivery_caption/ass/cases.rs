use super::super::*;
use veac_plan::ResolvedTextStyle;

pub(super) fn fallback(style: &mut ResolvedTextStyle) {
    style.fallback_fonts.push(style.font.clone());
}

pub(super) fn oblique(style: &mut ResolvedTextStyle) {
    style.font_style = FontStyle::Oblique;
}

pub(super) fn line_height(style: &mut ResolvedTextStyle) {
    style.line_height = 1.25;
}

pub(super) fn text_box(style: &mut ResolvedTextStyle) {
    style.layout.box_width_pixels = Some(200.0);
    style.layout.box_height_pixels = Some(80.0);
}

pub(super) fn writing_mode(style: &mut ResolvedTextStyle) {
    style.layout.writing_mode = TextWritingMode::VerticalRl;
}

pub(super) fn text_path(style: &mut ResolvedTextStyle) {
    style.path = Some(TextPath {
        points: vec![point(0.0, 0.0), point(100.0, 0.0)],
        start_offset: pixels(0.0),
        reverse: false,
        alignment: TextPathAlignment::Start,
    });
}

pub(super) fn background(style: &mut ResolvedTextStyle) {
    style.background = Some(TextBackground {
        color: Color {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 128,
        },
        padding_pixels: 4.0,
    });
}

pub(super) fn animation(style: &mut ResolvedTextStyle) {
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Whole,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::constant(1.0),
        stagger: RationalTime::new(0, 600).unwrap(),
    });
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
