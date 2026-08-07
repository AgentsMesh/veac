use tempfile::tempdir;

use super::super::support::*;
use super::{install, progress_curve};

#[test]
fn progress_bindings_drive_text_reveal_opacity_and_highlight_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("temporal-text-visibility.mp4");
    let mut project = project(false);
    let reveal = scalar_binding(&mut project, "text_reveal", 0.0, 1.0);
    let opacity = scalar_binding(&mut project, "text_opacity", 0.15, 1.0);
    let highlight = reveal.clone();
    let animation = TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform::default(),
        reveal: Animatable::Binding { binding_id: reveal },
        opacity: Animatable::Binding {
            binding_id: opacity,
        },
        stagger: time(0),
        highlight: Some(TextHighlightAnimation {
            fill: color(255, 24, 24),
            progress: Animatable::Binding {
                binding_id: highlight,
            },
        }),
    };
    add_scene(&mut project, animation);
    render(project, &font_assets(), &output);

    let early_frame = rgb_frame(&output, 0.1);
    let late_frame = rgb_frame(&output, 0.8);
    let early = frame_stats(&early_frame);
    let late = frame_stats(&late_frame);
    assert!(late.ratio > early.ratio * 2.0, "{early:?} -> {late:?}");
    assert!(late.energy > early.energy * 3.0, "{early:?} -> {late:?}");
    assert!(red_pixels(&late_frame) > red_pixels(&early_frame) + 12);
}

#[test]
fn progress_bindings_drive_text_position_scale_and_rotation_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("temporal-text-transform.mp4");
    let mut project = project(false);
    let position = install(
        &mut project,
        "text_position",
        "itm_temporal_text",
        TemporalType::Point,
        progress_curve(point(-20.0, 0.0), point(20.0, 0.0)),
        1,
    );
    let scale = install(
        &mut project,
        "text_scale",
        "itm_temporal_text",
        TemporalType::Vec2,
        progress_curve(vector(0.7, 0.7), vector(1.6, 0.9)),
        1,
    );
    let rotation = install(
        &mut project,
        "text_rotation",
        "itm_temporal_text",
        TemporalType::Angle,
        progress_curve(angle(0.0), angle(55.0)),
        1,
    );
    let animation = TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform {
            position_offset: Animatable::Binding {
                binding_id: position,
            },
            scale: Animatable::Binding { binding_id: scale },
            rotation_degrees: Animatable::Binding {
                binding_id: rotation,
            },
        },
        reveal: Animatable::constant(1.0),
        opacity: Animatable::constant(1.0),
        stagger: time(0),
        highlight: None,
    };
    add_scene(&mut project, animation);
    render(project, &font_assets(), &output);

    let early_frame = rgb_frame(&output, 0.1);
    let late_frame = rgb_frame(&output, 0.8);
    let early = frame_stats(&early_frame);
    let late = frame_stats(&late_frame);
    let early_bounds = lit_bounds(&early_frame);
    let late_bounds = lit_bounds(&late_frame);
    assert!(
        late.centroid_x > early.centroid_x + 24.0,
        "{early:?} -> {late:?}"
    );
    assert!(
        late_bounds.0 > early_bounds.0 + 5,
        "{early_bounds:?} -> {late_bounds:?}"
    );
}

fn scalar_binding(
    project: &mut ProjectEnvelope,
    name: &str,
    start: f64,
    end: f64,
) -> TemporalBindingId {
    install(
        project,
        name,
        "itm_temporal_text",
        TemporalType::Scalar,
        progress_curve(scalar(start), scalar(end)),
        1,
    )
}

fn add_scene(project: &mut ProjectEnvelope, animation: TextAnimation) {
    project.project.materials.push(material(
        "med_temporal_font",
        MaterialKind::Font,
        StreamChoice::Disabled,
        StreamChoice::Disabled,
    ));
    let style = TextStyle {
        font: FontRef::Material {
            material_id: MaterialId::new("med_temporal_font").unwrap(),
        },
        size_pixels: 20.0,
        color: color(245, 245, 245),
        animation: Some(animation),
        ..TextStyle::default()
    };
    let mut text = text_clip("itm_temporal_text", "VEAC", style, 0, 1_000);
    text.visual = Some(full_visual());
    project.project.sequences[0].tracks.extend([
        track(
            "trk_temporal_text_base",
            TrackKind::Video,
            0,
            vec![solid_clip(
                "itm_temporal_text_base",
                color(0, 0, 0),
                0,
                1_000,
            )],
        ),
        track("trk_temporal_text", TrackKind::Visual, 1, vec![text]),
    ]);
}

fn font_assets() -> BTreeMap<String, PathBuf> {
    BTreeMap::from([("med_temporal_font".to_owned(), font_fixture())])
}

fn scalar(value: f64) -> TemporalValue {
    TemporalValue::Scalar { value }
}

fn angle(degrees: f64) -> TemporalValue {
    TemporalValue::Angle { degrees }
}

fn vector(x: f64, y: f64) -> TemporalValue {
    TemporalValue::Vec2 {
        value: Vec2 { x, y },
    }
}

fn point(x: f64, y: f64) -> TemporalValue {
    TemporalValue::Point {
        value: Point {
            x: pixels(x),
            y: pixels(y),
        },
    }
}

fn red_pixels(frame: &[u8]) -> usize {
    frame
        .chunks_exact(3)
        .filter(|pixel| pixel[0] > 100 && pixel[0] > pixel[1].saturating_mul(2))
        .count()
}
