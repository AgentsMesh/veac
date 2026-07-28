use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{ass_script, bindings, emit_video_command, resolved, text_fixture};

#[test]
fn visible_overflow_preserves_surface_and_maps_anchor_into_the_layout_box() {
    let plan = visible_overflow_plan(FitMode::Contain, (120.0, 60.0));
    let graph = graph(&plan);
    let ass = ass_script(&graph);

    assert!(graph.contains("s=1920x1080"), "graph={graph}");
    assert!(!graph.contains("scale=120:60"), "graph={graph}");
    assert!(!graph.contains("overflowframev"), "graph={graph}");
    assert!(ass.contains("PlayResX: 1920\nPlayResY: 1080"), "ass={ass}");
    for marker in ["-0.46875*w", "-0.5277777777777778*h"] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
}

#[test]
fn visible_overflow_fits_the_layout_basis_instead_of_the_padded_surface() {
    for (fit, expected_x, expected_y, scaled) in [
        (FitMode::Fill, "iw*2", "ih*1", true),
        (FitMode::Contain, "", "", false),
        (FitMode::Cover, "iw*2", "ih*2", true),
    ] {
        let plan = visible_overflow_plan(fit, (240.0, 60.0));
        let graph = graph(&plan);
        assert_eq!(graph.contains("overflowframev"), scaled, "graph={graph}");
        if scaled {
            assert!(graph.contains(expected_x), "missing {expected_x}: {graph}");
            assert!(graph.contains(expected_y), "missing {expected_y}: {graph}");
        }
        assert!(!graph.contains("scale=240:60"), "graph={graph}");
    }
}

fn visible_overflow_plan(fit: FitMode, frame: (f64, f64)) -> veac_plan::ResolvedRenderPlan {
    let mut plan = resolved(&text_fixture(false));
    let clip = &mut plan.sequences[0].tracks[1].clips[0];
    let ResolvedClipSource::Text { content } = &mut clip.source else {
        panic!("text fixture")
    };
    content.style.layout = TextLayout {
        box_width_pixels: Some(120.0),
        box_height_pixels: Some(60.0),
        overflow: TextOverflow::Visible,
        ..TextLayout::default()
    };
    let visual = clip.visual.as_mut().unwrap();
    visual.frame = Some(Frame {
        width: pixels(frame.0),
        height: pixels(frame.1),
        fit,
    });
    visual.placement = Placement::Absolute {
        position: Point {
            x: pixels(700.0),
            y: pixels(400.0),
        },
    };
    visual.transform = Transform2D {
        position: Animatable::constant(Point {
            x: pixels(0.0),
            y: pixels(0.0),
        }),
        scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
        flip_horizontal: false,
        flip_vertical: false,
        rotation_degrees: Animatable::constant(0.0),
        anchor: Vec2 { x: 0.0, y: 1.0 },
        crop: None,
    };
    visual.opacity = Animatable::constant(1.0);
    visual.compositing = Compositing {
        z_index: 1,
        blend_mode: BlendMode::Normal,
    };
    visual.masks.clear();
    visual.card = None;
    plan
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}
