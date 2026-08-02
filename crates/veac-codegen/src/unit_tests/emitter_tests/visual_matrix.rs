use veac_codegen::emitter::geometry;
use veac_plan::canonical::*;

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn fill_contain_cover_and_no_frame_take_distinct_paths() {
    let cases = [
        (FitMode::Fill, "scale=320:180["),
        (FitMode::Contain, "force_original_aspect_ratio=decrease"),
        (FitMode::Cover, "force_original_aspect_ratio=increase"),
    ];
    for (fit, marker) in cases {
        let mut plan = resolved(&fixture());
        visual(&mut plan).frame = Some(Frame {
            width: Length {
                value: 320.0,
                unit: LengthUnit::Pixels,
            },
            height: Length {
                value: 180.0,
                unit: LengthUnit::Pixels,
            },
            fit,
        });
        assert!(graph(&plan).contains(marker));
    }
    let plan = resolved(&fixture());
    assert!(!graph(&plan).contains("framev"));
}

#[test]
fn every_anchor_and_absolute_length_unit_produces_geometry() {
    let mut plan = resolved(&fixture());
    let visual = visual(&mut plan);
    for anchor in [
        Anchor::TopLeft,
        Anchor::Top,
        Anchor::TopRight,
        Anchor::Left,
        Anchor::Center,
        Anchor::Right,
        Anchor::BottomLeft,
        Anchor::Bottom,
        Anchor::BottomRight,
    ] {
        visual.placement = Placement::Anchor {
            anchor,
            inset: Vec2 { x: 2.0, y: 3.0 },
        };
        let (x, y) = geometry::overlay_position(visual, "t", 0.5, 0.5);
        assert!(x.contains('2') && y.contains('3'), "{anchor:?}: {x}, {y}");
    }
    for unit in [
        LengthUnit::Pixels,
        LengthUnit::Normalized,
        LengthUnit::Percent,
    ] {
        visual.placement = Placement::Absolute {
            position: Point {
                x: Length { value: 25.0, unit },
                y: Length { value: 50.0, unit },
            },
        };
        let (x, y) = geometry::overlay_position(visual, "t", 0.5, 0.5);
        assert!(x.contains("25") && y.contains("50"), "{unit:?}: {x}, {y}");
    }
}

#[test]
fn identity_visual_properties_skip_optional_filters() {
    let mut plan = resolved(&fixture());
    let visual = visual(&mut plan);
    visual.transform.crop = None;
    visual.transform.scale = Animatable::constant(Vec2 { x: 1.0, y: 1.0 });
    visual.transform.rotation_degrees = Animatable::constant(0.0);
    visual.opacity = Animatable::constant(1.0);
    visual.card = Some(CardStyle {
        corner_radius_pixels: 0.0,
        shadow: None,
    });
    let graph = graph(&plan);
    for absent in ["cropv", "transformscalev", "pivotv", "opacityv", "cornerv"] {
        assert!(!graph.contains(absent), "unexpected {absent}: {graph}");
    }
}

#[test]
fn animated_crop_uses_a_fixed_viewport_with_per_frame_origin_and_size() {
    let mut plan = resolved(&fixture());
    visual(&mut plan).transform.crop = Some(Animatable::Keyframes {
        keyframes: vec![
            Keyframe {
                id: KeyframeId::new("kf_crop_a").unwrap(),
                time: time(0),
                value: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.5,
                    height: 1.0,
                },
                interpolation: Interpolation::Linear,
            },
            Keyframe {
                id: KeyframeId::new("kf_crop_b").unwrap(),
                time: time(300),
                value: Rect {
                    x: 0.5,
                    y: 0.1,
                    width: 0.4,
                    height: 0.8,
                },
                interpolation: Interpolation::Linear,
            },
        ],
    });
    let graph = graph(&plan);
    assert!(
        graph.contains("cropzoomv") && graph.contains(":eval=frame"),
        "{graph}"
    );
    assert!(
        graph.contains("crop=w='max(1\\,iw*0.5)':h='max(1\\,ih*1)'"),
        "{graph}"
    );
}

fn visual(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::EffectiveVisualProperties {
    plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}
