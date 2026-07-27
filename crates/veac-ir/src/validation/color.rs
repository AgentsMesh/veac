use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn color_pipeline(&mut self, value: &ColorPipeline, path: &str, id: &str) {
        if ![value.input, value.working, value.output]
            .into_iter()
            .all(color_space_valid)
        {
            self.value_error("COLOR_SPACE", path, id);
        }
        if value.stages.len() > 64 {
            self.value_error("COLOR_STAGE_COUNT", path, id);
        }
        for (index, stage) in value.stages.iter().enumerate() {
            let stage_path = format!("{path}/visual/color_pipeline/stages/{index}");
            match stage {
                ColorStage::Basic { adjustment } if !basic_valid(*adjustment) => {
                    self.value_error("COLOR_BASIC", &stage_path, id);
                }
                ColorStage::Matrix { adjustment } if !matrix_valid(*adjustment) => {
                    self.value_error("COLOR_MATRIX", &stage_path, id);
                }
                ColorStage::Hsl { adjustment } if !hsl_valid(*adjustment) => {
                    self.value_error("COLOR_HSL", &stage_path, id);
                }
                ColorStage::Curves { curves } if !curves_valid(curves) => {
                    self.value_error("COLOR_CURVES", &stage_path, id);
                }
                ColorStage::Wheels { wheels } if !wheels_valid(*wheels) => {
                    self.value_error("COLOR_WHEELS", &stage_path, id);
                }
                ColorStage::Lut { application } => {
                    self.lut(application, &stage_path, id);
                }
                _ => {}
            }
        }
    }

    fn lut(&mut self, value: &LutApplication, path: &str, id: &str) {
        let Some(kind) = self.material_ids.get(value.material_id.as_str()).copied() else {
            self.missing_ref("LUT_MATERIAL_NOT_FOUND", value.material_id.as_str(), path);
            return;
        };
        let valid = match kind {
            MaterialKind::Lut1d => matches!(
                value.interpolation,
                LutInterpolation::Nearest
                    | LutInterpolation::Linear
                    | LutInterpolation::Cosine
                    | LutInterpolation::Cubic
                    | LutInterpolation::Spline
            ),
            MaterialKind::Lut3d => matches!(
                value.interpolation,
                LutInterpolation::Nearest
                    | LutInterpolation::Trilinear
                    | LutInterpolation::Tetrahedral
                    | LutInterpolation::Pyramid
                    | LutInterpolation::Prism
            ),
            _ => false,
        };
        if !valid {
            self.value_error("LUT_MATERIAL_KIND", path, id);
        }
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
    let values = [&value.luma, &value.red, &value.green, &value.blue];
    values.iter().any(|curve| curve.is_some()) && values.into_iter().flatten().all(curve_valid)
}

fn curve_valid(value: &ToneCurve) -> bool {
    value.points.len() >= 2
        && value.points.len() <= 64
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

fn finite(value: f64, min: f64, max: f64) -> bool {
    value.is_finite() && (min..=max).contains(&value)
}
