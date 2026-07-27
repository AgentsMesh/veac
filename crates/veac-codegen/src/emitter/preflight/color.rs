use veac_plan::canonical::{
    BasicColorAdjustment, ColorCurves, HslAdjustment, LiftGammaGain, LutInterpolation,
    RgbMatrixAdjustment, ToneCurve,
};
use veac_plan::{
    ResolvedColorPipeline, ResolvedColorStage, ResolvedInputKind, ResolvedLut, ResolvedLutKind,
    ResolvedRenderPlan, ResolvedSequence,
};

use super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan, sequence: &ResolvedSequence) {
    for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
        let Some(pipeline) = clip
            .visual
            .as_ref()
            .and_then(|visual| visual.color_pipeline.as_ref())
        else {
            continue;
        };
        validate_pipeline(check, plan, &clip.id.to_string(), pipeline);
    }
}

pub(super) fn validate_pipeline(
    check: &mut Check,
    plan: &ResolvedRenderPlan,
    owner_id: &str,
    pipeline: &ResolvedColorPipeline,
) {
    let valid = [pipeline.input, pipeline.working, pipeline.output]
        .into_iter()
        .all(veac_plan::canonical::color_space_valid)
        && pipeline.stages.len() <= 64
        && pipeline.stages.iter().all(|stage| stage_valid(plan, stage));
    if !valid {
        check.push(
            "PLAN_COLOR_PIPELINE_INVALID",
            Some(owner_id.to_owned()),
            "color stages contain invalid values or unsupported resolved combinations",
        );
    }
}

fn stage_valid(plan: &ResolvedRenderPlan, value: &ResolvedColorStage) -> bool {
    match value {
        ResolvedColorStage::Basic { adjustment } => basic_valid(*adjustment),
        ResolvedColorStage::Matrix { adjustment } => matrix_valid(*adjustment),
        ResolvedColorStage::Hsl { adjustment } => hsl_valid(*adjustment),
        ResolvedColorStage::Curves { curves } => curves_valid(curves),
        ResolvedColorStage::Wheels { wheels } => wheels_valid(*wheels),
        ResolvedColorStage::Lut { application } => lut_valid(plan, application),
    }
}

fn basic_valid(value: BasicColorAdjustment) -> bool {
    finite(value.exposure_stops, -10.0, 10.0)
        && finite(value.temperature_kelvin, 1000.0, 40000.0)
        && finite(value.tint, -1.0, 1.0)
        && finite(value.highlights, -1.0, 1.0)
        && finite(value.shadows, -1.0, 1.0)
        && finite(value.fade, 0.0, 1.0)
}

fn matrix_valid(value: RgbMatrixAdjustment) -> bool {
    value
        .matrix
        .iter()
        .all(|component| finite(*component, -16.0, 16.0))
        && value
            .offset
            .iter()
            .all(|component| finite(*component, -4.0, 4.0))
}

fn hsl_valid(value: HslAdjustment) -> bool {
    finite(value.hue_degrees, -180.0, 180.0)
        && finite(value.saturation, -1.0, 1.0)
        && finite(value.lightness, -1.0, 1.0)
}

fn curves_valid(value: &ColorCurves) -> bool {
    let curves = [&value.luma, &value.red, &value.green, &value.blue];
    curves.iter().any(|curve| curve.is_some()) && curves.into_iter().flatten().all(curve_valid)
}

fn curve_valid(value: &ToneCurve) -> bool {
    (2..=64).contains(&value.points.len())
        && value
            .points
            .iter()
            .all(|point| finite(point.input, 0.0, 1.0) && finite(point.output, 0.0, 1.0))
        && value
            .points
            .windows(2)
            .all(|pair| pair[0].input < pair[1].input)
}

fn wheels_valid(value: LiftGammaGain) -> bool {
    [value.lift, value.gamma, value.gain]
        .into_iter()
        .flat_map(|wheel| [wheel.red, wheel.green, wheel.blue])
        .all(|component| finite(component, -1.0, 1.0))
}

fn lut_valid(plan: &ResolvedRenderPlan, value: &ResolvedLut) -> bool {
    let interpolation = matches!(
        (value.kind, value.interpolation),
        (
            ResolvedLutKind::OneDimensional,
            LutInterpolation::Nearest
                | LutInterpolation::Linear
                | LutInterpolation::Cosine
                | LutInterpolation::Cubic
                | LutInterpolation::Spline
        ) | (
            ResolvedLutKind::ThreeDimensional,
            LutInterpolation::Nearest
                | LutInterpolation::Trilinear
                | LutInterpolation::Tetrahedral
                | LutInterpolation::Pyramid
                | LutInterpolation::Prism
        )
    );
    let expected = match value.kind {
        ResolvedLutKind::OneDimensional => veac_plan::canonical::MaterialKind::Lut1d,
        ResolvedLutKind::ThreeDimensional => veac_plan::canonical::MaterialKind::Lut3d,
    };
    let mut inputs = plan
        .inputs
        .iter()
        .filter(|input| input.id == value.input_id);
    let kind = inputs.next().is_some_and(|input| {
        matches!(
            &input.kind,
            ResolvedInputKind::Resource { material_kind } if *material_kind == expected
        )
    });
    interpolation && kind && inputs.next().is_none()
}

fn finite(value: f64, min: f64, max: f64) -> bool {
    value.is_finite() && (min..=max).contains(&value)
}
