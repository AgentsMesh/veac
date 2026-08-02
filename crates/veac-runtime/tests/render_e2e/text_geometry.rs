use tempfile::tempdir;
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn vertical_writing_direction_and_orientation_change_real_pixel_layout() {
    let temp = tempdir().unwrap();
    let assets = font_assets();
    let (_, rl) = render_frame(
        temp.path(),
        "vertical-rl.mp4",
        "I\nMM",
        vertical_style(TextWritingMode::VerticalRl, TextOrientation::Upright),
        &assets,
    );
    let (_, lr) = render_frame(
        temp.path(),
        "vertical-lr.mp4",
        "I\nMM",
        vertical_style(TextWritingMode::VerticalLr, TextOrientation::Upright),
        &assets,
    );
    let (_, sideways) = render_frame(
        temp.path(),
        "vertical-sideways.mp4",
        "I\nMM",
        vertical_style(TextWritingMode::VerticalLr, TextOrientation::Sideways),
        &assets,
    );

    let rl_stats = frame_stats(&rl);
    let lr_stats = frame_stats(&lr);
    assert!(
        lr_stats.centroid_x > rl_stats.centroid_x + 5.0,
        "column order did not move pixels: rl={rl_stats:?}, lr={lr_stats:?}"
    );
    let upright_bounds = lit_bounds(&lr);
    let sideways_bounds = lit_bounds(&sideways);
    assert!(
        sideways_bounds.0 > upright_bounds.0,
        "sideways glyphs did not widen the layout: upright={upright_bounds:?}, sideways={sideways_bounds:?}"
    );
}

#[test]
fn text_path_unit_position_scale_and_rotation_change_real_pixels() {
    let temp = tempdir().unwrap();
    let assets = font_assets();
    let (output, _) = render_frame(
        temp.path(),
        "path-transformed.mp4",
        "I",
        path_style(TextUnitTransform {
            position_offset: curve("position", [point(0.0, 0.0), point(8.0, 6.0)]),
            scale: curve("scale", [Vec2 { x: 1.0, y: 1.0 }, Vec2 { x: 1.8, y: 0.7 }]),
            rotation_degrees: curve("rotation", [0.0, 70.0]),
        }),
        &assets,
    );
    let baseline = rgb_frame(&output, 0.05);
    let transformed = rgb_frame(&output, 0.85);

    let before = frame_stats(&baseline);
    let after = frame_stats(&transformed);
    let baseline_bounds = lit_bounds(&baseline);
    let transformed_bounds = lit_bounds(&transformed);
    assert!(
        after.centroid_x > before.centroid_x + 4.0 && after.centroid_y > before.centroid_y + 2.0,
        "unit position was not visible: before={before:?}, after={after:?}"
    );
    assert!(
        transformed_bounds.0 > baseline_bounds.0,
        "scale/rotation did not widen the glyph: baseline={baseline_bounds:?}, transformed={transformed_bounds:?}"
    );
}

fn render_frame(
    root: &Path,
    name: &str,
    text: &str,
    style: TextStyle,
    assets: &BTreeMap<String, PathBuf>,
) -> (PathBuf, Vec<u8>) {
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_font",
        MaterialKind::Font,
        StreamChoice::Disabled,
        StreamChoice::Disabled,
    ));
    let mut text = text_clip("itm_geometry", text, style, 0, 1_000);
    text.visual = Some(full_visual());
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_background",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_background", color(0, 0, 0), 0, 1_000)],
        ),
        track("trk_text", TrackKind::Visual, 1, vec![text]),
    ]);
    let output = root.join(name);
    render(canonical, assets, &output);
    assert_media_contract(&output, 0, 1.0);
    let frame = rgb_frame(&output, 0.5);
    (output, frame)
}

fn vertical_style(mode: TextWritingMode, orientation: TextOrientation) -> TextStyle {
    TextStyle {
        font: font_ref(),
        size_pixels: 16.0,
        color: color(255, 255, 255),
        layout: TextLayout {
            box_width_pixels: Some(64.0),
            box_height_pixels: Some(48.0),
            writing_mode: mode,
            orientation,
            ..TextLayout::default()
        },
        ..TextStyle::default()
    }
}

fn path_style(transform: TextUnitTransform) -> TextStyle {
    TextStyle {
        font: font_ref(),
        size_pixels: 18.0,
        color: color(255, 255, 255),
        path: Some(TextPath {
            points: vec![point(16.0, 27.0), point(80.0, 27.0)],
            start_offset: pixels(38.0),
            reverse: false,
            alignment: TextPathAlignment::Center,
        }),
        animation: Some(TextAnimation {
            granularity: TextGranularity::Grapheme,
            transform,
            reveal: Animatable::constant(1.0),
            opacity: Animatable::constant(1.0),
            stagger: time(0),
            highlight: None,
        }),
        ..TextStyle::default()
    }
}

fn font_ref() -> FontRef {
    FontRef::Material {
        material_id: MaterialId::new("med_font").unwrap(),
    }
}

fn font_assets() -> BTreeMap<String, PathBuf> {
    BTreeMap::from([("med_font".to_owned(), font_fixture())])
}

fn point(x: f64, y: f64) -> Point {
    Point {
        x: pixels(x),
        y: pixels(y),
    }
}

fn curve<T>(name: &str, values: [T; 2]) -> Animatable<T> {
    Animatable::Keyframes {
        keyframes: values
            .into_iter()
            .enumerate()
            .map(|(index, value)| Keyframe {
                id: KeyframeId::new(format!("kf_{name}_{index}")).unwrap(),
                time: time(index as i64 * 1_000),
                value,
                interpolation: Interpolation::Linear,
            })
            .collect(),
    }
}
