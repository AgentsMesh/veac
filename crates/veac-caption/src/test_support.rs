use std::collections::BTreeMap;

use veac_ir::{
    Animatable, BlendMode, Compositing, FontRef, Length, LengthUnit, Placement, Point,
    RationalTime, TextStyle, TimeRange, Transform2D, Vec2, VisualProperties,
};

use crate::*;

pub(crate) fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 1000).unwrap()
}

pub(crate) fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(time(start), time(duration)).unwrap()
}

pub(crate) fn style() -> CaptionStyle {
    CaptionStyle {
        id: "Default".to_owned(),
        font_family: "Inter".to_owned(),
        font_size_pixels: 42,
        foreground_color: "&H00FFFFFF".to_owned(),
        secondary_color: "&H000000FF".to_owned(),
        outline_color: "&H00000000".to_owned(),
        background_color: "&H80000000".to_owned(),
        bold: false,
        italic: false,
        underline: false,
        strikeout: false,
        scale_x_percent: 100.0,
        scale_y_percent: 100.0,
        letter_spacing_pixels: 0.0,
        rotation_degrees: 0.0,
        border_style: 1,
        outline_pixels: 2.0,
        shadow_pixels: 1.0,
        alignment: 2,
        margin_left: 10,
        margin_right: 10,
        margin_vertical: 10,
        encoding: 1,
    }
}

pub(crate) fn cue(id: &str, start: i64, text: &str) -> CaptionCue {
    CaptionCue {
        id: CaptionCueId::new(id).unwrap(),
        range: range(start, 1000),
        text: CaptionText {
            plain: text.to_owned(),
            spans: Vec::new(),
        },
        speaker: None,
        style: Some("Default".to_owned()),
        settings: BTreeMap::new(),
        words: Vec::new(),
    }
}

pub(crate) fn envelope() -> CaptionEnvelope {
    let mut document = CaptionDocument::new(1000, OverlapPolicy::Reject);
    document.language = Some("zh-Hans".to_owned());
    document.styles.push(style());
    document.cues = vec![
        cue("cap_one", 0, "你好\nVEAC"),
        cue("cap_two", 1500, "world"),
    ];
    CaptionEnvelope::new(document)
}

pub(crate) fn text_style() -> TextStyle {
    TextStyle {
        font: FontRef::Family {
            family: "Inter".to_owned(),
        },
        ..TextStyle::default()
    }
}

pub(crate) fn visual() -> VisualProperties {
    let point = Point {
        x: Length {
            value: 50.0,
            unit: LengthUnit::Percent,
        },
        y: Length {
            value: 90.0,
            unit: LengthUnit::Percent,
        },
    };
    VisualProperties {
        placement: Placement::Absolute { position: point },
        frame: None,
        transform: Transform2D {
            position: Animatable::constant(point),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 1.0 },
            crop: None,
        },
        opacity: Animatable::constant(1.0),
        compositing: Compositing {
            z_index: 20,
            blend_mode: BlendMode::Normal,
        },
        masks: vec![],
        card: None,
        color_pipeline: None,
    }
}
