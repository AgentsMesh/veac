use crate::*;

use super::time;

pub(crate) fn visual() -> VisualProperties {
    VisualProperties {
        placement: Placement::Anchor {
            anchor: Anchor::BottomRight,
            inset: Vec2 { x: 24.0, y: 36.0 },
        },
        frame: Some(Frame {
            width: Length {
                value: 320.0,
                unit: LengthUnit::Pixels,
            },
            height: Length {
                value: 180.0,
                unit: LengthUnit::Pixels,
            },
            fit: FitMode::Contain,
        }),
        transform: Transform2D {
            position: Animatable::constant(Point {
                x: Length {
                    value: 0.0,
                    unit: LengthUnit::Pixels,
                },
                y: Length {
                    value: 0.0,
                    unit: LengthUnit::Pixels,
                },
            }),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            shear: Vec2 { x: 0.0, y: 0.0 },
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: None,
        },
        opacity: Animatable::Keyframes {
            keyframes: vec![
                Keyframe {
                    id: KeyframeId::new("kf_opacity_start").unwrap(),
                    time: time(0),
                    value: 0.0,
                    interpolation: Interpolation::EaseOut,
                },
                Keyframe {
                    id: KeyframeId::new("kf_opacity_end").unwrap(),
                    time: time(60),
                    value: 1.0,
                    interpolation: Interpolation::Linear,
                },
            ],
        },
        compositing: Compositing {
            z_index: 10,
            blend_mode: BlendMode::Normal,
        },
        masks: vec![Mask {
            shape: MaskShape::Circle,
            position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            rotation_degrees: Animatable::constant(0.0),
            feather_pixels: Animatable::constant(4.0),
            expansion_pixels: Animatable::constant(0.0),
            invert: false,
        }],
        card: Some(CardStyle {
            corner_radius_pixels: 12.0,
            shadow: Some(Shadow {
                blur_pixels: 16.0,
                opacity: 0.4,
                offset: Vec2 { x: 2.0, y: 8.0 },
                color: Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                },
            }),
        }),
        color_pipeline: None,
    }
}

pub(crate) fn identity_layout_visual() -> VisualProperties {
    let mut value = visual();
    value.placement = Placement::Anchor {
        anchor: Anchor::Center,
        inset: Vec2 { x: 0.0, y: 0.0 },
    };
    value.frame = None;
    value.transform = Transform2D {
        position: Animatable::constant(Point {
            x: Length {
                value: 0.0,
                unit: LengthUnit::Pixels,
            },
            y: Length {
                value: 0.0,
                unit: LengthUnit::Pixels,
            },
        }),
        scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
        shear: Vec2 { x: 0.0, y: 0.0 },
        flip_horizontal: false,
        flip_vertical: false,
        rotation_degrees: Animatable::constant(0.0),
        anchor: Vec2 { x: 0.5, y: 0.5 },
        crop: None,
    };
    value.opacity = Animatable::constant(1.0);
    value.compositing = Compositing {
        z_index: 0,
        blend_mode: BlendMode::Normal,
    };
    value.masks.clear();
    value.card = None;
    value.color_pipeline = None;
    value
}
