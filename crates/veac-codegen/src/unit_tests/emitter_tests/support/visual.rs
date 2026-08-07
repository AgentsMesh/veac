use veac_plan::canonical::*;

use super::time;

pub fn visual() -> VisualProperties {
    VisualProperties {
        placement: Placement::Anchor {
            anchor: Anchor::BottomRight,
            inset: Vec2 { x: 24.0, y: 32.0 },
        },
        frame: Some(Frame {
            width: Length {
                value: 0.4,
                unit: LengthUnit::Normalized,
            },
            height: Length {
                value: 40.0,
                unit: LengthUnit::Percent,
            },
            fit: FitMode::Cover,
        }),
        transform: Transform2D {
            position: Animatable::Keyframes {
                keyframes: vec![
                    point_keyframe("kf_start", 0, 0.0),
                    point_keyframe("kf_end", 600, -50.0),
                ],
            },
            scale: Animatable::constant(Vec2 { x: 0.8, y: 0.9 }),
            shear: Vec2 { x: 0.0, y: 0.0 },
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(12.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: Some(Animatable::constant(Rect {
                x: 0.1,
                y: 0.1,
                width: 0.8,
                height: 0.8,
            })),
        },
        opacity: Animatable::Keyframes {
            keyframes: vec![
                number_keyframe("kf_opacity_a", 0, 0.0),
                number_keyframe("kf_opacity_b", 120, 1.0),
            ],
        },
        compositing: Compositing {
            z_index: 4,
            blend_mode: BlendMode::Screen,
        },
        masks: vec![Mask {
            shape: MaskShape::Ellipse,
            position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            rotation_degrees: Animatable::constant(0.0),
            feather_pixels: Animatable::constant(3.0),
            expansion_pixels: Animatable::constant(0.0),
            invert: false,
        }],
        card: Some(CardStyle {
            corner_radius_pixels: 16.0,
            shadow: Some(Shadow {
                blur_pixels: 8.0,
                opacity: 0.4,
                offset: Vec2 { x: 4.0, y: 5.0 },
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

pub fn transition_visual() -> VisualProperties {
    let mut value = visual();
    value.frame = None;
    value.transform.position = Animatable::constant(Point {
        x: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
    });
    value.opacity = Animatable::constant(1.0);
    value.compositing = Compositing {
        z_index: 0,
        blend_mode: BlendMode::Normal,
    };
    value.masks.clear();
    value.card = None;
    value
}

pub fn point_keyframe(id: &str, value: i64, x: f64) -> Keyframe<Point> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(value),
        value: Point {
            x: Length {
                value: x,
                unit: LengthUnit::Pixels,
            },
            y: Length {
                value: 0.0,
                unit: LengthUnit::Pixels,
            },
        },
        interpolation: Interpolation::EaseInOut,
    }
}

pub fn number_keyframe(id: &str, value: i64, number: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(value),
        value: number,
        interpolation: Interpolation::Linear,
    }
}

pub fn rekey_visual(value: &mut veac_plan::EffectiveVisualProperties, prefix: &str) {
    let mut index = 0;
    rekey(&mut value.transform.position, prefix, &mut index);
    rekey(&mut value.transform.scale, prefix, &mut index);
    rekey(&mut value.transform.rotation_degrees, prefix, &mut index);
    rekey(&mut value.opacity, prefix, &mut index);
    for mask in &mut value.masks {
        rekey(&mut mask.position, prefix, &mut index);
        rekey(&mut mask.scale, prefix, &mut index);
        rekey(&mut mask.rotation_degrees, prefix, &mut index);
        rekey(&mut mask.feather_pixels, prefix, &mut index);
        rekey(&mut mask.expansion_pixels, prefix, &mut index);
    }
}

fn rekey<T>(value: &mut Animatable<T>, prefix: &str, index: &mut usize) {
    let Animatable::Keyframes { keyframes } = value else {
        return;
    };
    for keyframe in keyframes {
        keyframe.id = KeyframeId::new(format!("kf_{prefix}_{index}")).unwrap();
        *index += 1;
    }
}
