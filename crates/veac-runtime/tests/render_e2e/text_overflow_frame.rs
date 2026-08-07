use tempfile::tempdir;
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn visible_overflow_frame_keeps_readable_glyphs_and_places_the_layout_pivot_once() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_font",
        MaterialKind::Font,
        StreamChoice::Disabled,
        StreamChoice::Disabled,
    ));
    let style = TextStyle {
        font: FontRef::Material {
            material_id: MaterialId::new("med_font").unwrap(),
        },
        size_pixels: 18.0,
        color: color(255, 255, 255),
        layout: TextLayout {
            box_width_pixels: Some(30.0),
            box_height_pixels: Some(24.0),
            overflow: TextOverflow::Visible,
            ..TextLayout::default()
        },
        ..TextStyle::default()
    };
    let mut text = text_clip("itm_overflow", "MMMM", style, 0, 1_000);
    text.visual = Some(overflow_visual());
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_background",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_background", color(0, 0, 0), 0, 1_000)],
        ),
        track("trk_text", TrackKind::Visual, 1, vec![text]),
    ]);

    let output = temp.path().join("text-overflow-frame.mp4");
    let assets = BTreeMap::from([("med_font".to_owned(), font_fixture())]);
    let rendered = render(canonical, &assets, &output);
    let graph = rendered.command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("s=96x54"), "graph={graph}");
    assert!(!graph.contains("scale=30:24"), "graph={graph}");
    assert_media_contract(&output, 0, 1.0);

    let frame = rgb_frame(&output, 0.5);
    let bounds = lit_bounds(&frame);
    let extents = lit_extents(&frame);
    assert!(bounds.0 > 35 && bounds.1 > 10, "bounds={bounds:?}");
    assert!(
        extents.left <= 50 && extents.right >= 50 && extents.top <= 28 && extents.bottom + 4 >= 28,
        "extents={extents:?}, bounds={bounds:?}"
    );
}

#[test]
fn visible_overflow_budget_uses_the_scaled_surface_not_the_frame_box() {
    let temp = tempdir().unwrap();
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
        color: color(255, 255, 255),
        ..TextStyle::default()
    };
    style.layout.box_width_pixels = Some(20.0);
    style.layout.box_height_pixels = Some(20.0);
    style.layout.overflow = TextOverflow::Visible;
    let mut text = text_clip("itm_scaled_surface", "M", style, 0, 500);
    let mut visual = full_visual();
    visual.frame = Some(Frame {
        width: pixels(40.0),
        height: pixels(40.0),
        fit: FitMode::Contain,
    });
    text.visual = Some(visual);
    canonical.project.sequences[0].tracks = vec![track(
        "trk_scaled_surface",
        TrackKind::Visual,
        0,
        vec![text],
    )];
    let output = temp.path().join("scaled-overflow-surface.mp4");
    let assets = BTreeMap::from([("med_font".to_owned(), font_fixture())]);
    let rendered = render(canonical, &assets, &output);

    let graph = rendered.command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("pad=w='192*2+96':h='108*2+54'"), "{graph}");
    assert!(frame_stats(&rgb_frame(&output, 0.25)).energy > 20.0);
}

#[test]
fn caption_visible_overflow_respects_absolute_bottom_placement() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_font",
        MaterialKind::Font,
        StreamChoice::Disabled,
        StreamChoice::Disabled,
    ));
    let style = TextStyle {
        font: FontRef::Material {
            material_id: MaterialId::new("med_font").unwrap(),
        },
        size_pixels: 8.0,
        color: color(255, 255, 255),
        layout: TextLayout {
            box_width_pixels: Some(80.0),
            box_height_pixels: Some(8.0),
            vertical_alignment: VerticalTextAlignment::Bottom,
            overflow: TextOverflow::Visible,
            ..TextLayout::default()
        },
        ..TextStyle::default()
    };
    let mut caption = text_clip("itm_caption_overflow", "CAPTION", style.clone(), 0, 1_000);
    caption.source = ClipSource::Caption {
        text: "CAPTION".to_owned(),
        speaker: None,
        cue: Box::default(),
        style,
    };
    let mut visual = full_visual();
    visual.frame = Some(Frame {
        width: pixels(80.0),
        height: pixels(8.0),
        fit: FitMode::Contain,
    });
    visual.placement = Placement::Absolute {
        position: Point {
            x: pixels(48.0),
            y: pixels(46.0),
        },
    };
    caption.visual = Some(visual);
    canonical.project.sequences[0].tracks = vec![track(
        "trk_caption_overflow",
        TrackKind::Caption,
        0,
        vec![caption],
    )];

    let output = temp.path().join("caption-overflow-frame.mp4");
    let assets = BTreeMap::from([("med_font".to_owned(), font_fixture())]);
    render(canonical, &assets, &output);
    let stats = frame_stats(&rgb_frame(&output, 0.5));
    assert!(
        stats.centroid_y > 42.0,
        "caption was not placed at the bottom: {stats:?}"
    );
}

fn overflow_visual() -> VisualProperties {
    let mut visual = full_visual();
    visual.frame = Some(Frame {
        width: pixels(30.0),
        height: pixels(24.0),
        fit: FitMode::Contain,
    });
    visual.placement = Placement::Absolute {
        position: Point {
            x: pixels(50.0),
            y: pixels(28.0),
        },
    };
    visual.transform.anchor = Vec2 { x: 0.25, y: 0.75 };
    visual.compositing.z_index = 1;
    visual
}
