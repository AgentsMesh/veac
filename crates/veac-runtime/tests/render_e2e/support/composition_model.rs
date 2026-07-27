use super::*;

pub(crate) fn full_visual() -> VisualProperties {
    framed_visual(Anchor::Center, None)
}

pub(crate) fn framed_visual(anchor: Anchor, size: Option<(f64, f64)>) -> VisualProperties {
    VisualProperties {
        placement: Placement::Anchor {
            anchor,
            inset: Vec2 { x: 0.0, y: 0.0 },
        },
        frame: size.map(|(width, height)| Frame {
            width: pixels(width),
            height: pixels(height),
            fit: FitMode::Fill,
        }),
        transform: Transform2D {
            position: Animatable::constant(Point {
                x: pixels(0.0),
                y: pixels(0.0),
            }),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: None,
        },
        opacity: Animatable::constant(1.0),
        compositing: Compositing {
            z_index: 0,
            blend_mode: BlendMode::Normal,
        },
        masks: vec![],
        card: None,
        color_pipeline: None,
    }
}

pub(crate) fn default_mask(shape: MaskShape) -> Mask {
    Mask {
        shape,
        position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
        scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
        rotation_degrees: Animatable::constant(0.0),
        feather_pixels: Animatable::constant(0.0),
        expansion_pixels: Animatable::constant(0.0),
        invert: false,
    }
}

pub(crate) fn video_effect(
    id: &str,
    effect_type: &str,
    parameters: BTreeMap<String, ParameterValue>,
) -> EffectInstance {
    EffectInstance {
        id: EffectId::new(id).unwrap(),
        effect_type: effect_type.to_owned(),
        enabled: true,
        enable_range: None,
        parameters,
    }
}

pub(crate) fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}
