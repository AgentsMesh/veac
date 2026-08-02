use veac_codegen::emitter::emit_all;
use veac_plan::canonical::{
    Animatable, CardStyle, Color, FitMode, Frame, Length, LengthUnit, Rect, Shadow, Vec2,
    MAX_SHADOW_BLUR_PIXELS, MIN_CROP_EXTENT,
};

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn shadow_blur_budget_is_enforced_before_filter_construction() {
    let mut plan = resolved(&fixture());
    plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .card = Some(CardStyle {
        corner_radius_pixels: 0.0,
        shadow: Some(Shadow {
            blur_pixels: MAX_SHADOW_BLUR_PIXELS + 1.0,
            opacity: 0.5,
            offset: Vec2 { x: 0.0, y: 0.0 },
            color: Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
        }),
    });

    let error = emit_all(&plan, &bindings(&plan)).unwrap_err();

    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "PLAN_VISUAL_INVALID"));
}

#[test]
fn subpixel_frames_and_crops_quantize_to_nonzero_filter_geometry() {
    let mut plan = resolved(&fixture());
    let visual = plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap();
    visual.frame = Some(Frame {
        width: Length {
            value: 0.1,
            unit: LengthUnit::Pixels,
        },
        height: Length {
            value: 0.1,
            unit: LengthUnit::Pixels,
        },
        fit: FitMode::Fill,
    });
    visual.transform.crop = Some(Animatable::constant(Rect {
        x: 0.0,
        y: 0.0,
        width: MIN_CROP_EXTENT,
        height: MIN_CROP_EXTENT,
    }));

    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();

    assert!(graph.contains("max(1\\,iw*0.015625)"), "{graph}");
    assert!(graph.contains("scale=1:1"), "{graph}");
}
