use tempfile::tempdir;
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn alpha_and_inverted_luma_mattes_control_cross_clip_visibility() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("track-mattes.mp4");
    let mut project = project(false);
    let background = solid_clip("itm_matte_bg", color(0, 0, 255), 0, 2_000);
    let alpha_source = alpha_source();
    let luma_source = luma_source();
    let alpha_target = target("itm_alpha_target", 0);
    let luma_target = target("itm_luma_target", 1_000);
    project.project.sequences[0].tracks.extend([
        track("trk_matte_bg", TrackKind::Video, 0, vec![background]),
        track(
            "trk_matte_targets",
            TrackKind::Visual,
            1,
            vec![alpha_target, luma_target],
        ),
        track(
            "trk_matte_sources",
            TrackKind::Visual,
            2,
            vec![alpha_source, luma_source],
        ),
    ]);
    add_matte(
        &mut project,
        "seq_main",
        "itm_alpha_source",
        "itm_alpha_target",
        TrackMatteMode::Alpha,
        false,
    );
    add_matte(
        &mut project,
        "seq_main",
        "itm_luma_source",
        "itm_luma_target",
        TrackMatteMode::Luma,
        true,
    );

    let rendered = render(project, &BTreeMap::new(), &output);
    let graph = rendered.command.filter_graph.unwrap();
    assert_red(rgb_at(&output, 0.5, WIDTH / 2, HEIGHT / 2));
    assert_blue(rgb_at(&output, 0.5, 8, 8));
    let dark_side = rgb_at(&output, 1.5, 14, HEIGHT / 2);
    let light_side = rgb_at(&output, 1.5, 82, HEIGHT / 2);
    assert!(
        i16::from(dark_side[0]) > i16::from(dark_side[2]) + 70,
        "dark={dark_side:?}"
    );
    assert!(
        i16::from(light_side[2]) > i16::from(light_side[0]) + 70,
        "light={light_side:?}\n{graph}"
    );
}

#[test]
fn text_layer_can_drive_a_real_alpha_matte() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("text-matte.mp4");
    let mut project = project(false);
    project.project.materials.push(material(
        "med_matte_font",
        MaterialKind::Font,
        StreamChoice::Disabled,
        StreamChoice::Disabled,
    ));
    let background = solid_clip("itm_text_matte_bg", color(0, 0, 220), 0, 1_000);
    let mut target = solid_clip("itm_text_matte_target", color(230, 0, 0), 0, 1_000);
    target.visual = Some(full_visual());
    let mut source = text_clip(
        "itm_text_matte_source",
        "MASK",
        matte_text_style(),
        0,
        1_000,
    );
    source.visual = Some(full_visual());
    project.project.sequences[0].tracks.extend([
        track("trk_text_matte_bg", TrackKind::Video, 0, vec![background]),
        track("trk_text_matte_target", TrackKind::Visual, 1, vec![target]),
        track("trk_text_matte_source", TrackKind::Visual, 2, vec![source]),
    ]);
    add_matte(
        &mut project,
        "seq_main",
        "itm_text_matte_source",
        "itm_text_matte_target",
        TrackMatteMode::Alpha,
        false,
    );
    let assets = BTreeMap::from([("med_matte_font".to_owned(), font_fixture())]);

    let rendered = render(project, &assets, &output);
    let graph = rendered.command.filter_graph.unwrap();
    assert!(graph.contains("textassv") && graph.contains("mattetrimv"));
    let frame = rgb_frame(&output, 0.5);
    let red = frame
        .chunks_exact(3)
        .filter(|pixel| pixel[0] > 120 && pixel[0] > pixel[2] + 50)
        .count();
    let blue = frame
        .chunks_exact(3)
        .filter(|pixel| pixel[2] > 120 && pixel[2] > pixel[0] + 50)
        .count();
    assert!(red > 20, "matte did not reveal text: red={red}");
    assert!(
        blue > 3_000,
        "matte did not preserve background: blue={blue}"
    );
}

fn matte_text_style() -> TextStyle {
    TextStyle {
        font: FontRef::Material {
            material_id: MaterialId::new("med_matte_font").unwrap(),
        },
        size_pixels: 30.0,
        color: color(255, 255, 255),
        ..TextStyle::default()
    }
}

fn alpha_source() -> Clip {
    let mut clip = solid_clip("itm_alpha_source", color(255, 255, 255), 0, 1_000);
    let mut visual = full_visual();
    let mut mask = default_mask(MaskShape::Circle);
    mask.scale = Animatable::constant(Vec2 { x: 0.6, y: 0.6 });
    visual.masks.push(mask);
    clip.visual = Some(visual);
    clip
}

fn luma_source() -> Clip {
    let mut clip = solid_clip("itm_luma_source", color(0, 0, 0), 1_000, 1_000);
    clip.source = ClipSource::Generated {
        generator: Generator::Gradient {
            gradient: Gradient::Linear {
                start: Vec2 { x: 0.0, y: 0.5 },
                end: Vec2 { x: 1.0, y: 0.5 },
                stops: vec![
                    GradientStop {
                        offset: 0.0,
                        color: color(0, 0, 0),
                    },
                    GradientStop {
                        offset: 1.0,
                        color: color(255, 255, 255),
                    },
                ],
            },
        },
    };
    clip.visual = Some(full_visual());
    clip
}

fn target(id: &str, start: i64) -> Clip {
    let mut clip = solid_clip(id, color(255, 0, 0), start, 1_000);
    clip.visual = Some(full_visual());
    clip
}

fn assert_red(pixel: [u8; 3]) {
    assert!(pixel[0] > 170 && pixel[2] < 70, "pixel={pixel:?}");
}

fn assert_blue(pixel: [u8; 3]) {
    assert!(pixel[2] > 170 && pixel[0] < 70, "pixel={pixel:?}");
}
