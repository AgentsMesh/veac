use tempfile::tempdir;
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn text_uses_the_common_animated_composition_pipeline() {
    let temp = tempdir().unwrap();
    let font = font_fixture();
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_font",
        MaterialKind::Font,
        StreamChoice::Disabled,
        StreamChoice::Disabled,
    ));
    let mut text = text_clip("itm_text", "VEAC", text_style(), 0, 2_000);
    text.visual = Some(text_visual());
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_text_background",
            TrackKind::Video,
            0,
            vec![solid_clip(
                "itm_text_background",
                color(10, 40, 80),
                0,
                2_000,
            )],
        ),
        track("trk_text", TrackKind::Visual, 1, vec![text]),
    ]);
    let assets = BTreeMap::from([("med_font".to_owned(), font)]);
    let output = temp.path().join("text.mp4");
    let rendered = render(canonical, &assets, &output);

    assert_eq!(rendered.plan.inputs.len(), 1);
    assert!(
        rendered.command.inputs.is_empty(),
        "fonts are bindings, not media inputs"
    );
    let graph = rendered.command.filter_graph.as_deref().unwrap();
    for mechanism in [
        "subtitles=filename=",
        "textassv",
        "cornerv",
        "maskv",
        "rotate=",
        "opacityv",
        "shadowv",
        "blend=all_mode=screen",
    ] {
        assert!(graph.contains(mechanism), "missing {mechanism}: {graph}");
    }
    assert_media_contract(&output, 0, 2.0);
    let early = frame_stats_with_threshold(&rgb_frame(&output, 0.2), 12);
    let middle = frame_stats_with_threshold(&rgb_frame(&output, 1.0), 12);
    let late = frame_stats_with_threshold(&rgb_frame(&output, 1.8), 12);
    assert!(
        early.ratio > 0.01 && middle.ratio > 0.01 && late.ratio > 0.01,
        "early={early:?}, middle={middle:?}, late={late:?}"
    );
    assert!(early.centroid_x < 40.0, "early={early:?}");
    assert!(
        (40.0..=56.0).contains(&middle.centroid_x),
        "middle={middle:?}"
    );
    assert!(late.centroid_x > 56.0, "late={late:?}");
    assert!(
        middle.energy > early.energy * 1.15,
        "{early:?} -> {middle:?}"
    );
    assert!(middle.energy > late.energy * 1.08, "{middle:?} -> {late:?}");
    assert!(early.right_strip_ratio < 0.03, "early={early:?}");
    assert!(late.left_strip_ratio < 0.03, "late={late:?}");
}

fn text_style() -> TextStyle {
    TextStyle {
        font: FontRef::Material {
            material_id: MaterialId::new("med_font").unwrap(),
        },
        size_pixels: 18.0,
        color: color(255, 240, 40),
        background: Some(TextBackground {
            color: Color {
                alpha: 180,
                ..color(20, 20, 20)
            },
            padding_pixels: 2.0,
        }),
        outline: Some(TextOutline {
            color: color(255, 255, 255),
            width_pixels: 1.0,
        }),
        shadow: Some(shadow(2.0, 0.65, 2.0, 2.0)),
        ..TextStyle::default()
    }
}

fn text_visual() -> VisualProperties {
    VisualProperties {
        placement: Placement::Anchor {
            anchor: Anchor::Center,
            inset: Vec2 { x: 0.0, y: 0.0 },
        },
        frame: Some(Frame {
            width: pixels(40.0),
            height: pixels(24.0),
            fit: FitMode::Fill,
        }),
        transform: Transform2D {
            position: point_curve(),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            shear: Vec2 { x: 0.0, y: 0.0 },
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(8.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: None,
        },
        opacity: number_curve([0.35, 1.0, 0.55], "opacity"),
        compositing: Compositing {
            z_index: 1,
            blend_mode: BlendMode::Screen,
        },
        masks: vec![Mask {
            shape: MaskShape::Rectangle,
            position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            rotation_degrees: Animatable::constant(0.0),
            feather_pixels: Animatable::constant(0.5),
            expansion_pixels: Animatable::constant(0.0),
            invert: false,
        }],
        card: Some(CardStyle {
            corner_radius_pixels: 3.0,
            shadow: Some(shadow(2.0, 0.55, 2.0, 2.0)),
        }),
        color_pipeline: None,
    }
}

fn point_curve() -> Animatable<Point> {
    Animatable::Keyframes {
        keyframes: [-24.0, 0.0, 24.0]
            .into_iter()
            .enumerate()
            .map(|(index, x)| Keyframe {
                id: KeyframeId::new(format!("kf_text_position_{index}")).unwrap(),
                time: time(index as i64 * 1_000),
                value: Point {
                    x: pixels(x),
                    y: pixels(0.0),
                },
                interpolation: Interpolation::Linear,
            })
            .collect(),
    }
}

fn number_curve(values: [f64; 3], name: &str) -> Animatable<f64> {
    Animatable::Keyframes {
        keyframes: values
            .into_iter()
            .enumerate()
            .map(|(index, value)| Keyframe {
                id: KeyframeId::new(format!("kf_text_{name}_{index}")).unwrap(),
                time: time(index as i64 * 1_000),
                value,
                interpolation: Interpolation::Linear,
            })
            .collect(),
    }
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}

fn shadow(blur: f64, opacity: f64, x: f64, y: f64) -> Shadow {
    Shadow {
        blur_pixels: blur,
        opacity,
        offset: Vec2 { x, y },
        color: color(0, 0, 0),
    }
}
