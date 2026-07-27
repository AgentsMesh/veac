use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn canvas_generated_identity_skips_lossy_spatial_overlay() {
    let mut plan = identity_plan();
    let identity_graph = graph(&plan);
    assert!(
        identity_graph.contains("layerprecisionv"),
        "graph={identity_graph}"
    );
    assert!(
        identity_graph.contains("gradientcanvasv") && identity_graph.contains("format=gbrap16le"),
        "graph={identity_graph}"
    );
    assert!(identity_graph.contains("65535"), "graph={identity_graph}");
    assert!(
        !identity_graph.contains("layercanvasv"),
        "graph={identity_graph}"
    );

    clip(&mut plan).visual.as_mut().unwrap().transform.position = Animatable::constant(Point {
        x: pixels(1.0),
        y: pixels(0.0),
    });
    let translated_graph = graph(&plan);
    assert!(
        translated_graph.contains("layercanvasv"),
        "graph={translated_graph}"
    );
    assert!(
        translated_graph.contains("overlay=x="),
        "graph={translated_graph}"
    );
}

fn identity_plan() -> veac_plan::ResolvedRenderPlan {
    let mut plan = resolved(&fixture());
    let clip = clip(&mut plan);
    clip.source = ResolvedClipSource::Generated {
        generator: Generator::Gradient {
            gradient: Gradient::Linear {
                start: Vec2 { x: 0.0, y: 0.5 },
                end: Vec2 { x: 1.0, y: 0.5 },
                stops: vec![stop(0.0, 0), stop(1.0, 255)],
            },
        },
    };
    clip.effects.clear();
    let visual = clip.visual.as_mut().unwrap();
    visual.placement = Placement::Anchor {
        anchor: Anchor::Center,
        inset: Vec2 { x: 0.0, y: 0.0 },
    };
    visual.frame = None;
    visual.transform.position = Animatable::constant(Point {
        x: pixels(0.0),
        y: pixels(0.0),
    });
    visual.transform.scale = Animatable::constant(Vec2 { x: 1.0, y: 1.0 });
    visual.transform.flip_horizontal = false;
    visual.transform.flip_vertical = false;
    visual.transform.rotation_degrees = Animatable::constant(0.0);
    visual.transform.anchor = Vec2 { x: 0.5, y: 0.5 };
    visual.transform.crop = None;
    visual.opacity = Animatable::constant(1.0);
    visual.masks.clear();
    visual.compositing.blend_mode = BlendMode::Normal;
    visual.card = None;
    plan
}

fn clip(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences[0].tracks[0].clips[0]
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}

fn stop(offset: f64, value: u8) -> GradientStop {
    GradientStop {
        offset,
        color: Color {
            red: value,
            green: value,
            blue: value,
            alpha: 255,
        },
    }
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}
