use tempfile::tempdir;
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn gradients_and_vector_fill_stroke_render_expected_pixels() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    let mut gradient = solid_clip("itm_gradient", color(0, 0, 0), 0, 1_000);
    gradient.source = ClipSource::Generated {
        generator: Generator::Gradient {
            gradient: Gradient::Linear {
                start: Vec2 { x: 0.0, y: 0.5 },
                end: Vec2 { x: 1.0, y: 0.5 },
                stops: vec![
                    stop(0.0, color(255, 0, 0)),
                    stop(0.5, color(80, 0, 80)),
                    stop(1.0, color(0, 0, 255)),
                ],
            },
        },
    };
    let mut shape = solid_clip("itm_shape", color(0, 0, 0), 0, 1_000);
    shape.source = ClipSource::Generated {
        generator: Generator::Shape {
            shape: VectorShape {
                geometry: VectorGeometry::Rectangle {
                    bounds: Rect {
                        x: 0.25,
                        y: 0.2,
                        width: 0.5,
                        height: 0.6,
                    },
                },
                fill: Some(Paint::Solid {
                    color: color(0, 255, 0),
                }),
                stroke: Some(VectorStroke {
                    paint: Paint::Solid {
                        color: color(255, 255, 255),
                    },
                    width_pixels: 4.0,
                }),
            },
        },
    };
    canonical.project.sequences[0].tracks.extend([
        track("trk_gradient", TrackKind::Video, 0, vec![gradient]),
        track("trk_shape", TrackKind::Visual, 1, vec![shape]),
    ]);
    let output = temp.path().join("graphics.mp4");
    let rendered = render(canonical, &BTreeMap::new(), &output);
    let graph = rendered.command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("gradientv") && graph.contains("shapev"));
    assert_media_contract(&output, 0, 1.0);

    let left = rgb_at(&output, 0.5, 4, 27);
    let right = rgb_at(&output, 0.5, 90, 27);
    let center = rgb_at(&output, 0.5, 48, 27);
    let border = rgb_at(&output, 0.5, 25, 27);
    assert!(left[0] > left[2] * 2, "left={left:?}");
    assert!(right[2] > right[0] * 2, "right={right:?}");
    assert!(center[1] > 180 && center[0] < 80, "center={center:?}");
    assert!(border.iter().all(|value| *value > 170), "border={border:?}");
}

#[test]
fn text_box_right_bottom_alignment_and_whole_opacity_render_in_ffmpeg() {
    let temp = tempdir().unwrap();
    let font = font_fixture();
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_font",
        MaterialKind::Font,
        StreamChoice::Disabled,
        StreamChoice::Disabled,
    ));
    let mut style = TextStyle {
        font: FontRef::Material {
            material_id: MaterialId::new("med_font").unwrap(),
        },
        size_pixels: 18.0,
        layout: TextLayout {
            box_width_pixels: Some(60.0),
            box_height_pixels: Some(36.0),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            horizontal_alignment: HorizontalTextAlignment::Right,
            vertical_alignment: VerticalTextAlignment::Bottom,
            ..TextLayout::default()
        },
        line_height: 1.25,
        ..TextStyle::default()
    };
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Whole,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        opacity: Animatable::Keyframes {
            keyframes: vec![
                alpha_key("kf_alpha_start", 0, 0.0),
                alpha_key("kf_alpha_end", 500, 1.0),
            ],
        },
        stagger: time(0),
        highlight: None,
    });
    let mut text = text_clip("itm_text_layout", "I", style, 0, 1_000);
    text.visual = Some(text_visual());
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_text_bg",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_text_bg", color(0, 0, 0), 0, 1_000)],
        ),
        track("trk_text_layout", TrackKind::Visual, 1, vec![text]),
    ]);
    let output = temp.path().join("text-layout.mp4");
    let assets = BTreeMap::from([("med_font".to_owned(), font)]);
    let rendered = render(canonical, &assets, &output);
    let graph = rendered.command.filter_graph.as_deref().unwrap();
    for marker in ["s=60x36", "subtitles=filename=", "textassv"] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
    let early = light_stats(&rgb_frame(&output, 0.1));
    let late = light_stats(&rgb_frame(&output, 0.8));
    assert!(late.0 > early.0 * 2.0, "early={early:?}, late={late:?}");
    assert!(late.1 > 65.0 && late.2 > 24.0, "late={late:?}");
}

fn stop(offset: f64, color: Color) -> GradientStop {
    GradientStop { offset, color }
}

fn alpha_key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Linear,
    }
}

fn text_visual() -> VisualProperties {
    let mut visual = full_visual();
    visual.compositing.z_index = 1;
    visual
}

fn light_stats(frame: &[u8]) -> (f64, f64, f64) {
    let points: Vec<_> = frame
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, pixel)| pixel.iter().map(|value| usize::from(*value)).sum::<usize>() > 150)
        .map(|(index, _)| {
            (
                (index % WIDTH as usize) as f64,
                (index / WIDTH as usize) as f64,
            )
        })
        .collect();
    let count = points.len().max(1) as f64;
    (
        points.len() as f64,
        points.iter().map(|point| point.0).sum::<f64>() / count,
        points.iter().map(|point| point.1).sum::<f64>() / count,
    )
}
