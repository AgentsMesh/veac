use veac_ir::{Anchor, Clip, FitMode, MaskShape, Placement, ProjectEnvelope, Vec2};

use crate::support::{clip_by_key, lower_example};

type MaskPredicate = fn(&MaskShape) -> bool;

fn clip<'a>(project: &'a ProjectEnvelope, id: &str) -> &'a Clip {
    clip_by_key(project, id)
}

fn assert_anchor(project: &ProjectEnvelope, id: &str, expected: Anchor, inset: Vec2, pivot: Vec2) {
    let visual = clip(project, id)
        .visual
        .as_ref()
        .unwrap_or_else(|| panic!("{id} should have visual properties"));
    assert_eq!(
        visual.placement,
        Placement::Anchor {
            anchor: expected,
            inset,
        }
    );
    assert_eq!(visual.transform.anchor, pivot);
}

fn assert_frame(example: &str, id: &str, width: f64, height: f64, fit: FitMode) {
    let project = lower_example(example);
    let frame = clip(&project, id)
        .visual
        .as_ref()
        .and_then(|visual| visual.frame)
        .unwrap_or_else(|| panic!("{id} should have a layout frame"));
    assert_eq!((frame.width.value, frame.height.value), (width, height));
    assert_eq!(frame.fit, fit);
}

#[test]
fn positioned_examples_align_content_with_their_canvas_anchor() {
    let card = lower_example("card-overlay/main.veac");
    assert_anchor(
        &card,
        "panel",
        Anchor::Center,
        Vec2 { x: 0.0, y: 0.0 },
        Vec2 { x: 0.5, y: 0.5 },
    );
    assert_frame(
        "card-overlay/main.veac",
        "panel",
        720.0,
        380.0,
        FitMode::Fill,
    );

    let text = lower_example("text-layout/main.veac");
    assert_anchor(
        &text,
        "horizontal",
        Anchor::TopLeft,
        Vec2 { x: 32.0, y: 32.0 },
        Vec2 { x: 0.0, y: 0.0 },
    );
    assert_anchor(
        &text,
        "vertical-rl",
        Anchor::TopRight,
        Vec2 { x: 48.0, y: 48.0 },
        Vec2 { x: 1.0, y: 0.0 },
    );
    assert_anchor(
        &text,
        "vertical-lr",
        Anchor::TopLeft,
        Vec2 { x: 48.0, y: 48.0 },
        Vec2 { x: 0.0, y: 0.0 },
    );
    assert_anchor(
        &text,
        "word-wrap",
        Anchor::Bottom,
        Vec2 { x: 0.0, y: 32.0 },
        Vec2 { x: 0.5, y: 1.0 },
    );
}

#[test]
fn transformed_badge_uses_the_same_corner_for_placement_and_pivot() {
    let project = lower_example("transforms-and-animation/main.veac");
    assert_anchor(
        &project,
        "badge",
        Anchor::BottomRight,
        Vec2 { x: 80.0, y: 80.0 },
        Vec2 { x: 1.0, y: 1.0 },
    );
}

#[test]
fn audio_example_has_a_persistent_visual_explanation() {
    let project = lower_example("audio-processing/main.veac");
    let plate = clip(&project, "plate");
    assert_eq!(plate.record_range.start.value, 0);
    assert_eq!(plate.record_range.duration.value, 2_400);
    for (key, start) in [("voice-label", 0), ("route-label", 1_200)] {
        let value = clip(&project, key);
        assert_eq!(value.record_range.start.value, start);
        assert_eq!(value.record_range.duration.value, 1_200);
    }
}

#[test]
fn fixture_media_is_scaled_to_the_composition_canvas() {
    for (example, id) in [
        ("speed-demo/main.veac", "fast"),
        ("speed-demo/main.veac", "slow"),
        ("nested-and-multicam/main.veac", "interview-cut"),
    ] {
        assert_frame(example, id, 1280.0, 720.0, FitMode::Contain);
    }
    for id in ["linear-trim", "speed-ramp", "freeze", "fast-forward"] {
        assert_frame(
            "timeline-source-time/main.veac",
            id,
            640.0,
            360.0,
            FitMode::Contain,
        );
    }
    assert_frame(
        "template-fill/main.veac",
        "hero",
        1280.0,
        720.0,
        FitMode::Cover,
    );
    assert_frame(
        "all-features/main.veac",
        "shot",
        1920.0,
        1080.0,
        FitMode::Cover,
    );
    assert_frame(
        "all-features/main.veac",
        "lower-third",
        1500.0,
        180.0,
        FitMode::Fill,
    );
}

#[test]
fn mask_gallery_exposes_each_geometry_before_composing_them() {
    let project = lower_example("masks-and-mattes/main.veac");
    let cases: [(&str, MaskPredicate); 6] = [
        ("circle", |shape| matches!(shape, MaskShape::Circle)),
        ("rectangle", |shape| matches!(shape, MaskShape::Rectangle)),
        ("ellipse", |shape| matches!(shape, MaskShape::Ellipse)),
        ("rounded", |shape| {
            matches!(shape, MaskShape::RoundedRectangle { .. })
        }),
        ("polygon", |shape| {
            matches!(shape, MaskShape::Polygon { .. })
        }),
        ("path", |shape| matches!(shape, MaskShape::Path { .. })),
    ];
    for (id, predicate) in cases {
        let masks = &clip(&project, id).visual.as_ref().expect("visual").masks;
        assert_eq!(masks.len(), 1, "{id} should isolate one mask");
        assert!(predicate(&masks[0].shape), "unexpected mask on {id}");
    }
    assert_eq!(
        clip(&project, "combined")
            .visual
            .as_ref()
            .expect("combined mask visual")
            .masks
            .len(),
        6
    );
}
