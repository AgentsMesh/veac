use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{bindings, emit_video_command, fixture, point_keyframe, resolved};

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
        !identity_graph.contains("layerplacedv"),
        "graph={identity_graph}"
    );

    clip(&mut plan).visual.as_mut().unwrap().transform.position = Animatable::constant(Point {
        x: pixels(1.0),
        y: pixels(0.0),
    });
    let translated_graph = graph(&plan);
    assert!(
        translated_graph.contains("layerplacedv")
            && translated_graph.contains("pad=w='1920*2+1920':h='1080*2+1080'")
            && translated_graph.contains("clip(")
            && translated_graph.contains("color=black@0:eval=frame")
            && translated_graph.contains("crop=w=1920:h=1080:x=1920:y=1080"),
        "graph={translated_graph}"
    );
    assert!(
        !translated_graph.contains("placement8v")
            && !translated_graph.contains("format=rgba,overlay"),
        "graph={translated_graph}"
    );

    let visual = clip(&mut plan).visual.as_mut().unwrap();
    visual.transform.position = Animatable::constant(Point {
        x: pixels(0.0),
        y: pixels(0.0),
    });
    visual.transform.shear = Vec2 { x: 0.25, y: 0.0 };
    let sheared_graph = graph(&plan);
    assert!(
        sheared_graph.contains("shearv") && sheared_graph.contains("layerplacedv"),
        "graph={sheared_graph}"
    );
    assert!(!sheared_graph.contains("placement8v"), "{sheared_graph}");
}

#[test]
fn large_generated_canvas_identity_is_not_charged_for_an_unemitted_pad() {
    let mut plan = identity_plan();
    plan.sequences[0].settings.width = 4_096;
    plan.sequences[0].settings.height = 4_096;

    let graph = graph(&plan);
    assert!(!graph.contains("layerplacedv"), "graph={graph}");
}

#[test]
fn framed_generated_source_is_resized_and_placed_on_the_canvas() {
    let mut plan = identity_plan();
    clip(&mut plan).visual.as_mut().unwrap().frame = Some(Frame {
        width: pixels(320.0),
        height: pixels(180.0),
        fit: FitMode::Fill,
    });

    let graph = graph(&plan);
    assert!(graph.contains("scale=320:180[framev"), "graph={graph}");
    assert!(graph.contains("layerplacedv"), "graph={graph}");
}

#[test]
fn animated_position_is_not_charged_for_the_static_pad_path() {
    let mut plan = identity_plan();
    plan.sequences[0].settings.width = 4_096;
    plan.sequences[0].settings.height = 4_096;
    plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .transform
        .position = Animatable::Keyframes {
        keyframes: vec![
            point_keyframe("kf_placement_left", 0, -100.0),
            point_keyframe("kf_placement_right", 500, 100.0),
        ],
    };

    let graph = graph(&plan);
    assert!(graph.contains("placement10v"), "graph={graph}");
    assert!(
        graph.contains("format=auto:alpha=straight") && !graph.contains("format=rgba,"),
        "graph={graph}"
    );
    assert!(!graph.contains("layerplacedv"), "graph={graph}");
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
    visual.transform.shear = Vec2 { x: 0.0, y: 0.0 };
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
