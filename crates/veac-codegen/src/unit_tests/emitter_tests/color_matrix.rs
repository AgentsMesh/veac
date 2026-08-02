use veac_plan::canonical::RgbMatrixAdjustment;
use veac_plan::ResolvedColorStage;

use super::support::{bindings, emit_video_command, graded_project, resolved};
use veac_plan::canonical::{LutInterpolation, MaterialKind};

#[test]
fn affine_rgb_matrix_uses_one_float_domain_geq_and_preserves_alpha() {
    let mut plan = resolved(&graded_project(
        MaterialKind::Lut3d,
        LutInterpolation::Tetrahedral,
    ));
    let pipeline = plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline
        .as_mut()
        .unwrap();
    pipeline.stages.insert(
        0,
        ResolvedColorStage::Matrix {
            adjustment: RgbMatrixAdjustment {
                matrix: [0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.2, 0.3, 0.5],
                offset: [0.1, -0.2, 0.25],
            },
        },
    );
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    for marker in [
        "format=gbrapf32le",
        "r(X\\,Y)*0+g(X\\,Y)*1+b(X\\,Y)*0+0.1",
        "r(X\\,Y)*1+g(X\\,Y)*0+b(X\\,Y)*0+-0.2",
        "a='alpha(X\\,Y)'",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
    assert_eq!(graph.matches("format=gbrapf32le[matrixfmtv").count(), 1);
}
