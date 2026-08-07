use veac_ir::{
    Anchor, Animatable, BlendMode, Compositing, Length, LengthUnit, Placement, Point, Transform2D,
    Vec2, VisualProperties,
};

pub(super) fn visual(z_index: i32) -> VisualProperties {
    let zero = Length {
        value: 0.0,
        unit: LengthUnit::Pixels,
    };
    VisualProperties {
        placement: Placement::Anchor {
            anchor: Anchor::Center,
            inset: Vec2 { x: 0.0, y: 0.0 },
        },
        frame: None,
        transform: Transform2D {
            position: Animatable::constant(Point { x: zero, y: zero }),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            shear: Vec2 { x: 0.0, y: 0.0 },
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: None,
        },
        opacity: Animatable::constant(1.0),
        compositing: Compositing {
            z_index,
            blend_mode: BlendMode::Normal,
        },
        masks: Vec::new(),
        card: None,
        color_pipeline: None,
    }
}
