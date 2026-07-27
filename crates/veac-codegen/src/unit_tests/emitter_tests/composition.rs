use veac_plan::canonical::*;

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn every_mask_shape_and_inversion_emits_alpha_math() {
    let mut plan = resolved(&fixture());
    let visual = clip(&mut plan).visual.as_mut().unwrap();
    visual.masks = vec![
        mask(MaskShape::Linear, false, 1.0),
        mask(MaskShape::Linear, true, 0.0),
        mask(MaskShape::Mirror, false, 0.0),
        mask(MaskShape::Mirror, true, 0.0),
        mask(MaskShape::Circle, false, 0.0),
        mask(MaskShape::Rectangle, true, 0.0),
        mask(MaskShape::Ellipse, false, 0.0),
        mask(MaskShape::Heart, true, 0.0),
        mask(MaskShape::Star, false, 0.0),
        mask(
            MaskShape::Path {
                points: vec![
                    Vec2 { x: 0.1, y: 0.1 },
                    Vec2 { x: 0.9, y: 0.1 },
                    Vec2 { x: 0.5, y: 0.9 },
                ],
            },
            false,
            2.0,
        ),
    ];
    visual.masks[0].position = Animatable::Keyframes {
        keyframes: vec![Keyframe {
            id: KeyframeId::new("kf_mask_position").unwrap(),
            time: RationalTime::new(0, 600).unwrap(),
            value: Vec2 { x: 0.4, y: 0.6 },
            interpolation: Interpolation::Linear,
        }],
    };
    visual.masks[0].rotation_degrees = Animatable::Keyframes {
        keyframes: vec![Keyframe {
            id: KeyframeId::new("kf_mask_rotation").unwrap(),
            time: RationalTime::new(0, 600).unwrap(),
            value: 15.0,
            interpolation: Interpolation::Linear,
        }],
    };
    let graph = graph(&plan);
    for marker in [
        "X/W",
        "X/W-",
        "0.5-abs",
        "0.5-hypot",
        "min(0.5-abs",
        "pow(pow",
        "cos(5*atan2",
        "mod(",
        "max(2*(1)",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
    assert!(graph.contains("if(lte(T\\,0)\\,0.4"));
    assert!(graph.contains("if(lte(T\\,0)\\,15"));
    assert!(graph.contains("enable='gte(t,0)*lt(t,1)'"));
    assert!(!graph.contains("enable='between(t,"));
}

#[test]
fn every_blend_mode_selects_the_backend_primitive() {
    let cases = [
        (BlendMode::Normal, None),
        (BlendMode::Multiply, Some("multiply")),
        (BlendMode::Screen, Some("screen")),
        (BlendMode::Overlay, Some("overlay")),
        (BlendMode::Darken, Some("darken")),
        (BlendMode::Lighten, Some("lighten")),
        (BlendMode::ColorDodge, Some("dodge")),
        (BlendMode::ColorBurn, Some("burn")),
        (BlendMode::HardLight, Some("hardlight")),
        (BlendMode::SoftLight, Some("softlight")),
        (BlendMode::Difference, Some("difference")),
        (BlendMode::Exclusion, Some("exclusion")),
    ];
    for (mode, backend) in cases {
        let mut plan = resolved(&fixture());
        clip(&mut plan)
            .visual
            .as_mut()
            .unwrap()
            .compositing
            .blend_mode = mode;
        let graph = graph(&plan);
        match backend {
            Some(name) => assert!(graph.contains(&format!("blend=all_mode={name}"))),
            None => assert!(!graph.contains("blend=all_mode=")),
        }
    }
}

#[test]
fn delayed_blend_source_is_expanded_to_the_timeline_only_once() {
    let mut plan = resolved(&fixture());
    let clip = clip(&mut plan);
    clip.record_range = TimeRange::new(
        RationalTime::new(300, 600).unwrap(),
        clip.record_range.duration,
    )
    .unwrap();
    clip.visual.as_mut().unwrap().compositing.blend_mode = BlendMode::Darken;
    plan.sequences[0].duration = RationalTime::new(900, 600).unwrap();

    let graph = graph(&plan);
    assert!(graph.contains("setpts=PTS+0.5/TB"), "graph={graph}");
    assert_eq!(graph.matches("overtimelinev").count(), 0, "graph={graph}");
    assert!(graph.contains("gte(T\\,0.5)*lt(T\\,1.5)"), "graph={graph}");
}

#[test]
fn horizontal_vertical_and_combined_flips_use_typed_filters() {
    for (horizontal, vertical, expected) in [
        (true, false, "hflip"),
        (false, true, "vflip"),
        (true, true, "hflip,vflip"),
    ] {
        let mut plan = resolved(&fixture());
        let transform = &mut clip(&mut plan).visual.as_mut().unwrap().transform;
        transform.flip_horizontal = horizontal;
        transform.flip_vertical = vertical;
        let graph = graph(&plan);
        assert!(graph.contains(expected), "missing {expected}: {graph}");
    }
}

fn mask(shape: MaskShape, invert: bool, feather_pixels: f64) -> Mask {
    Mask {
        shape,
        position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
        scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
        rotation_degrees: Animatable::constant(0.0),
        invert,
        feather_pixels: Animatable::constant(feather_pixels),
        expansion_pixels: Animatable::constant(0.0),
    }
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
