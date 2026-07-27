use super::*;

#[test]
fn visual_text_and_audio_values_are_validated_at_boundaries() {
    let mut project = sample_project();
    let video = &mut project.project.sequences[0].tracks[0].clips[0];
    let visual = video.visual.as_mut().unwrap();
    visual.placement = Placement::Anchor {
        anchor: Anchor::Center,
        inset: Vec2 { x: -1.0, y: 0.0 },
    };
    visual.frame.as_mut().unwrap().width.value = 0.0;
    visual.transform.position = Animatable::constant(Point {
        x: Length {
            value: f64::NAN,
            unit: LengthUnit::Normalized,
        },
        y: Length {
            value: 0.0,
            unit: LengthUnit::Normalized,
        },
    });
    visual.transform.scale = Animatable::constant(Vec2 { x: 0.0, y: 1.0 });
    visual.transform.rotation_degrees = Animatable::constant(f64::INFINITY);
    visual.transform.anchor = Vec2 { x: 2.0, y: 0.5 };
    visual.transform.crop = Some(Animatable::constant(Rect {
        x: 0.0,
        y: 0.8,
        width: 0.2,
        height: 0.4,
    }));
    visual.opacity = Animatable::constant(1.1);
    visual.masks[0].feather_pixels = Animatable::constant(-1.0);
    let card = visual.card.as_mut().unwrap();
    card.corner_radius_pixels = -1.0;
    let shadow = card.shadow.as_mut().unwrap();
    shadow.blur_pixels = MAX_SHADOW_BLUR_PIXELS + 1.0;
    shadow.opacity = 2.0;
    shadow.offset.x = f64::NAN;
    video.audio.as_mut().unwrap().gain = Animatable::constant(-1.0);
    video.audio.as_mut().unwrap().pan = Animatable::constant(2.0);

    let caption = &mut project.project.sequences[0].tracks[1].clips[0];
    if let ClipSource::Caption { style, .. } = &mut caption.source {
        style.size_pixels = 0.0;
        style.font = FontRef::Family {
            family: String::new(),
        };
        style.background.as_mut().unwrap().padding_pixels = -1.0;
        style.outline.as_mut().unwrap().width_pixels = -1.0;
        style.shadow = Some(Shadow {
            blur_pixels: 1.0,
            opacity: 2.0,
            offset: Vec2 {
                x: f64::NAN,
                y: 0.0,
            },
            color: Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
        });
    }
    let codes = validation_codes(&project);
    for code in [
        "PLACEMENT",
        "FRAME",
        "ANIMATION_VALUE",
        "ANCHOR",
        "CROP",
        "CARD",
        "SHADOW",
        "TEXT_STYLE",
    ] {
        assert_code(&codes, code);
    }
}
