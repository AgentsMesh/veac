use super::*;

#[test]
fn affine_rgb_matrix_validates_and_round_trips_canonically() {
    let project = matrix_project(RgbMatrixAdjustment {
        matrix: [0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        offset: [0.1, -0.1, 0.25],
    });
    validate(&project).unwrap();
    let json = canonical_json(&project).unwrap();
    assert_eq!(decode_canonical_json(&json).unwrap(), project);
}

#[test]
fn affine_rgb_matrix_rejects_nonfinite_and_unbounded_components() {
    for adjustment in [
        RgbMatrixAdjustment {
            matrix: [f64::NAN; 9],
            offset: [0.0; 3],
        },
        RgbMatrixAdjustment {
            matrix: [17.0; 9],
            offset: [0.0; 3],
        },
        RgbMatrixAdjustment {
            matrix: [1.0; 9],
            offset: [5.0; 3],
        },
    ] {
        assert_code(
            &validation_codes(&matrix_project(adjustment)),
            "COLOR_MATRIX",
        );
    }
}

fn matrix_project(adjustment: RgbMatrixAdjustment) -> ProjectEnvelope {
    let mut project = sample_project();
    let visual = project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap();
    visual.color_pipeline = Some(ColorPipeline {
        input: rec709(),
        working: rec709(),
        output: rec709(),
        stages: vec![ColorStage::Matrix { adjustment }],
    });
    project
}

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}
