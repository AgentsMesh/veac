use veac_ir::{Anchor, Animatable, Clip, FitMode, MaskShape, Placement, ProjectEnvelope, Vec2};

use crate::support::{clips, entry_sequence, lower_example};

type MaskPredicate = fn(&MaskShape) -> bool;

fn clip<'a>(project: &'a ProjectEnvelope, id: &str) -> &'a Clip {
    clips(project)
        .find(|clip| clip.id.as_str() == id)
        .unwrap_or_else(|| panic!("missing clip {id}"))
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
        "itm_panel",
        Anchor::Center,
        Vec2 { x: 0.0, y: 0.0 },
        Vec2 { x: 0.5, y: 0.5 },
    );
    assert_frame(
        "card-overlay/main.veac",
        "itm_panel",
        720.0,
        380.0,
        FitMode::Fill,
    );

    let text = lower_example("text-layout/main.veac");
    assert_anchor(
        &text,
        "itm_paragraph",
        Anchor::TopLeft,
        Vec2 { x: 48.0, y: 72.0 },
        Vec2 { x: 0.0, y: 0.0 },
    );
    assert_anchor(
        &text,
        "itm_vertical-label",
        Anchor::TopRight,
        Vec2 { x: 72.0, y: 72.0 },
        Vec2 { x: 1.0, y: 0.0 },
    );
    assert_anchor(
        &text,
        "itm_vertical-left-label",
        Anchor::BottomLeft,
        Vec2 { x: 72.0, y: 72.0 },
        Vec2 { x: 0.0, y: 1.0 },
    );
}

#[test]
fn transformed_badge_uses_the_same_corner_for_placement_and_pivot() {
    let project = lower_example("transforms-and-animation/main.veac");
    assert_anchor(
        &project,
        "itm_badge",
        Anchor::BottomRight,
        Vec2 { x: 80.0, y: 80.0 },
        Vec2 { x: 1.0, y: 1.0 },
    );
}

#[test]
fn audio_example_has_a_moving_visual_monitor() {
    let project = lower_example("audio-processing/main.veac");
    let monitor = entry_sequence(&project)
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_monitor")
        .expect("monitor visual layer");
    assert_eq!(monitor.clips.len(), 3);

    let playhead = clip(&project, "itm_processing-playhead")
        .visual
        .as_ref()
        .expect("playhead visual properties");
    let Animatable::Keyframes { keyframes } = &playhead.transform.position else {
        panic!("processing playhead position should be animated");
    };
    assert_eq!(keyframes.len(), 2);
    assert_eq!(keyframes[0].value.x.value, -474.0);
    assert_eq!(keyframes[1].value.x.value, 474.0);
}

#[test]
fn fixture_media_is_scaled_to_the_composition_canvas() {
    for (example, id) in [
        ("speed-demo/main.veac", "itm_fast"),
        ("speed-demo/main.veac", "itm_slow"),
        ("timeline-source-time/main.veac", "itm_linear-trim"),
        ("timeline-source-time/main.veac", "itm_speed-ramp"),
        ("timeline-source-time/main.veac", "itm_freeze"),
        ("timeline-source-time/main.veac", "itm_fast-forward"),
        ("nested-and-multicam/main.veac", "itm_interview-cut"),
    ] {
        assert_frame(example, id, 1280.0, 720.0, FitMode::Contain);
    }
    assert_frame(
        "template-fill/main.veac",
        "itm_hero",
        1280.0,
        720.0,
        FitMode::Cover,
    );
    assert_frame(
        "all-features/main.veac",
        "itm_shot",
        1920.0,
        1080.0,
        FitMode::Cover,
    );
    assert_frame(
        "all-features/main.veac",
        "itm_lower-third",
        1500.0,
        180.0,
        FitMode::Fill,
    );
}

#[test]
fn mask_gallery_exposes_each_geometry_before_composing_them() {
    let project = lower_example("masks-and-mattes/main.veac");
    let cases: [(&str, MaskPredicate); 6] = [
        ("itm_circle-stage", |shape| {
            matches!(shape, MaskShape::Circle)
        }),
        ("itm_rectangle-stage", |shape| {
            matches!(shape, MaskShape::Rectangle)
        }),
        ("itm_ellipse-stage", |shape| {
            matches!(shape, MaskShape::Ellipse)
        }),
        ("itm_rounded-stage", |shape| {
            matches!(shape, MaskShape::RoundedRectangle { .. })
        }),
        ("itm_polygon-stage", |shape| {
            matches!(shape, MaskShape::Polygon { .. })
        }),
        ("itm_path-stage", |shape| {
            matches!(shape, MaskShape::Path { .. })
        }),
    ];
    for (id, predicate) in cases {
        let masks = &clip(&project, id).visual.as_ref().expect("visual").masks;
        assert_eq!(masks.len(), 1, "{id} should isolate one mask");
        assert!(predicate(&masks[0].shape), "unexpected mask on {id}");
    }
    assert_eq!(
        clip(&project, "itm_combined-masks")
            .visual
            .as_ref()
            .expect("combined mask visual")
            .masks
            .len(),
        6
    );
}
