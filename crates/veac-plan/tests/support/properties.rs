use veac_plan::canonical::*;

pub fn audio_properties() -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    }
}

pub fn visual_properties() -> VisualProperties {
    let center = Point {
        x: Length {
            value: 0.5,
            unit: LengthUnit::Normalized,
        },
        y: Length {
            value: 0.5,
            unit: LengthUnit::Normalized,
        },
    };
    VisualProperties {
        placement: Placement::Anchor {
            anchor: Anchor::Center,
            inset: Vec2 { x: 0.0, y: 0.0 },
        },
        frame: None,
        transform: Transform2D {
            position: Animatable::constant(center),
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
            z_index: 0,
            blend_mode: BlendMode::Normal,
        },
        masks: Vec::new(),
        card: None,
        color_pipeline: None,
    }
}

pub fn text_style(font: FontRef) -> TextStyle {
    TextStyle {
        font,
        size_pixels: 42.0,
        color: Color {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        },
        background: None,
        outline: None,
        shadow: None,
        ..TextStyle::default()
    }
}
