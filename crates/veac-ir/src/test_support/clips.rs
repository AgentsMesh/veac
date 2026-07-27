use std::collections::BTreeMap;

use crate::*;

use super::{range, time, visual};

pub(crate) fn media_clip() -> Clip {
    Clip {
        id: ItemId::new("itm_video").unwrap(),
        enabled: true,
        record_range: range(0, 600),
        source: ClipSource::Media {
            material_id: MaterialId::new("med_video").unwrap(),
        },
        source_mapping: Some(SourceMapping::linear(time(0), Rational::new(1, 1).unwrap())),
        visual: Some(visual()),
        audio: Some(AudioProperties {
            gain: Animatable::constant(1.0),
            pan: Animatable::constant(0.0),
            muted: false,
            normalize: false,
            pitch_policy: PitchPolicy::Preserve,
            processors: vec![],
            crossfade: None,
        }),
        effects: vec![EffectInstance {
            id: EffectId::new("fx_color").unwrap(),
            effect_type: "video.color_adjust".to_owned(),
            enabled: true,
            enable_range: Some(range(0, 300)),
            parameters: BTreeMap::from([(
                "brightness".to_owned(),
                ParameterValue::Number { value: 0.2 },
            )]),
        }],
        replaceable: None,
        template_editable_text: false,
        metadata: BTreeMap::new(),
    }
}

pub(super) fn caption_clip(font_id: MaterialId) -> Clip {
    let white = Color {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    };
    let black = Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    Clip {
        id: ItemId::new("itm_caption").unwrap(),
        enabled: true,
        record_range: range(0, 300),
        source: ClipSource::Caption {
            text: "hello".to_owned(),
            speaker: None,
            style: TextStyle {
                font: FontRef::Material {
                    material_id: font_id,
                },
                size_pixels: 48.0,
                color: white,
                background: Some(TextBackground {
                    color: Color {
                        alpha: 128,
                        ..black
                    },
                    padding_pixels: 12.0,
                }),
                outline: Some(TextOutline {
                    color: black,
                    width_pixels: 2.0,
                }),
                shadow: None,
                ..TextStyle::default()
            },
        },
        source_mapping: None,
        visual: Some(caption_visual()),
        audio: None,
        effects: vec![],
        replaceable: None,
        template_editable_text: false,
        metadata: BTreeMap::new(),
    }
}

fn caption_visual() -> VisualProperties {
    let position = Point {
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
        opacity: Animatable::constant(1.0),
        transform: Transform2D {
            position: Animatable::constant(position),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 1.0 },
            crop: None,
        },
        placement: Placement::Absolute { position },
        frame: None,
        compositing: Compositing {
            z_index: 20,
            blend_mode: BlendMode::Normal,
        },
        masks: vec![],
        card: None,
        color_pipeline: None,
    }
}
