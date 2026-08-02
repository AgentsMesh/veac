use std::panic::{catch_unwind, AssertUnwindSafe};

use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::canonical::*;
use veac_plan::{PlanInputId, ResolvedColorStage, ResolvedInputKind, ResolvedRenderPlan};

use super::support::{bindings, emit_video_command, graded_project, resolved, tone_curve};

#[test]
fn every_malformed_resolved_tone_curve_fails_before_filter_generation() {
    let mut one_point = tone_curve(ToneCurveInterpolation::Natural);
    one_point.points.truncate(1);
    let mut too_many = tone_curve(ToneCurveInterpolation::Monotonic);
    too_many.points = (0..=64)
        .map(|index| CurvePoint {
            input: f64::from(index) / 64.0,
            output: f64::from(index) / 64.0,
        })
        .collect();
    let mut unordered = tone_curve(ToneCurveInterpolation::Natural);
    unordered.points[1].input = 0.0;
    let mut non_finite = tone_curve(ToneCurveInterpolation::Monotonic);
    non_finite.points[1].output = f64::NAN;
    let mut out_of_range = tone_curve(ToneCurveInterpolation::Natural);
    out_of_range.points[1].output = 1.1;

    let cases = [
        (
            "empty",
            ColorCurves {
                luma: None,
                red: None,
                green: None,
                blue: None,
            },
        ),
        ("one point", curves(one_point)),
        ("too many", curves(too_many)),
        ("unordered", curves(unordered)),
        ("non-finite", curves(non_finite)),
        ("out of range", curves(out_of_range)),
    ];
    for (name, value) in cases {
        let mut plan = color_plan();
        *curve_stage(&mut plan) = value;
        assert_invalid(plan, name);
    }
}

#[test]
fn color_stage_limit_and_non_curve_numeric_corruption_fail_closed() {
    let mut too_many = color_plan();
    let color_pipeline = pipeline(&mut too_many);
    color_pipeline.stages = std::iter::repeat_n(color_pipeline.stages[0].clone(), 65).collect();
    assert_invalid(too_many, "stage limit");

    let mut basic = color_plan();
    let ResolvedColorStage::Basic { adjustment } = &mut pipeline(&mut basic).stages[0] else {
        unreachable!()
    };
    adjustment.exposure_stops = f64::INFINITY;
    assert_invalid(basic, "basic number");

    let mut hsl = color_plan();
    let ResolvedColorStage::Hsl { adjustment } = &mut pipeline(&mut hsl).stages[1] else {
        unreachable!()
    };
    adjustment.saturation = 2.0;
    assert_invalid(hsl, "HSL number");

    let mut matrix = color_plan();
    pipeline(&mut matrix).stages.insert(
        0,
        ResolvedColorStage::Matrix {
            adjustment: RgbMatrixAdjustment {
                matrix: [f64::NAN, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
                offset: [0.0; 3],
            },
        },
    );
    assert_invalid(matrix, "matrix number");

    let mut wheels = color_plan();
    let ResolvedColorStage::Wheels { wheels: value } = &mut pipeline(&mut wheels).stages[3] else {
        unreachable!()
    };
    value.gain.blue = f64::NAN;
    assert_invalid(wheels, "wheel number");
}

#[test]
fn lut_input_identity_and_dimension_are_revalidated() {
    let mut missing = color_plan();
    let ResolvedColorStage::Lut { application } = &mut pipeline(&mut missing).stages[4] else {
        unreachable!()
    };
    application.input_id = PlanInputId::new("pin_missing_lut").unwrap();
    assert_invalid(missing, "missing LUT input");

    let mut wrong_kind = color_plan();
    let ResolvedColorStage::Lut { application } = &pipeline(&mut wrong_kind).stages[4] else {
        unreachable!()
    };
    let id = application.input_id.clone();
    wrong_kind
        .inputs
        .iter_mut()
        .find(|input| input.id == id)
        .unwrap()
        .kind = ResolvedInputKind::Resource {
        material_kind: MaterialKind::Lut1d,
    };
    assert_invalid(wrong_kind, "wrong LUT dimension");
}

fn curves(red: ToneCurve) -> ColorCurves {
    ColorCurves {
        luma: None,
        red: Some(red),
        green: None,
        blue: None,
    }
}

fn color_plan() -> ResolvedRenderPlan {
    resolved(&graded_project(
        MaterialKind::Lut3d,
        LutInterpolation::Tetrahedral,
    ))
}

fn pipeline(plan: &mut ResolvedRenderPlan) -> &mut veac_plan::ResolvedColorPipeline {
    plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline
        .as_mut()
        .unwrap()
}

fn curve_stage(plan: &mut ResolvedRenderPlan) -> &mut ColorCurves {
    let ResolvedColorStage::Curves { curves } = &mut pipeline(plan).stages[2] else {
        unreachable!()
    };
    curves
}

fn assert_invalid(plan: ResolvedRenderPlan, name: &str) {
    let local = bindings(&plan);
    let result = catch_unwind(AssertUnwindSafe(|| emit_video_command(&plan, &local)))
        .unwrap_or_else(|_| panic!("{name} panicked"));
    let error = result.expect_err(name);
    let diagnostic = error
        .diagnostics()
        .iter()
        .find(|value| value.code == "PLAN_COLOR_PIPELINE_INVALID")
        .unwrap_or_else(|| panic!("{name} returned {error}"));
    assert_eq!(diagnostic.kind, CodegenErrorKind::InvalidPlan, "{name}");
}
