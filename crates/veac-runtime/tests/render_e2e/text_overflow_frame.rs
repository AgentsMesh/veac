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
    let stats = frame_stats(&frame);
    assert!(bounds.0 > 35 && bounds.1 > 10, "bounds={bounds:?}");
    assert!(
        (48.0..=57.0).contains(&stats.centroid_x) && (17.0..=28.0).contains(&stats.centroid_y),
        "stats={stats:?}, bounds={bounds:?}"
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
