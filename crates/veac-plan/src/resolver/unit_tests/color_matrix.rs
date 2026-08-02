use super::support::*;
use crate::{canonical::*, resolve, ResolvedColorStage};

#[test]
fn affine_rgb_matrix_is_preserved_exactly_in_the_render_plan() {
    let adjustment = RgbMatrixAdjustment {
        matrix: [0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.25, 0.5, 0.25],
        offset: [0.1, -0.2, 0.3],
    };
    let mut value = project();
    let mut visual = visual_properties();
    visual.color_pipeline = Some(ColorPipeline {
        input: rec709(),
        working: rec709(),
        output: rec709(),
        stages: vec![ColorStage::Matrix { adjustment }],
    });
    value.project.sequences[0].tracks[0].clips[0].visual = Some(visual);
    let plan = resolve(&value, None).unwrap().remove(0);
    let stages = &plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .color_pipeline
        .as_ref()
        .unwrap()
        .stages;
    assert!(matches!(
        stages.as_slice(),
        [ResolvedColorStage::Matrix { adjustment: actual }] if *actual == adjustment
    ));
}

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}
