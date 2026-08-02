use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ColorSpace, MaterialId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColorPipeline {
    pub input: ColorSpace,
    pub working: ColorSpace,
    pub output: ColorSpace,
    pub stages: Vec<ColorStage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ColorStage {
    Basic { adjustment: BasicColorAdjustment },
    Matrix { adjustment: RgbMatrixAdjustment },
    Hsl { adjustment: HslAdjustment },
    Curves { curves: ColorCurves },
    Wheels { wheels: LiftGammaGain },
    Lut { application: LutApplication },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RgbMatrixAdjustment {
    /// Row-major RGB coefficients applied in the current working color space.
    pub matrix: [f64; 9],
    /// Normalized RGB offsets applied before the final `[0, 1]` clamp.
    pub offset: [f64; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BasicColorAdjustment {
    pub exposure_stops: f64,
    pub temperature_kelvin: f64,
    pub tint: f64,
    pub highlights: f64,
    pub shadows: f64,
    pub fade: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HueRange {
    Red,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HslAdjustment {
    pub range: HueRange,
    pub hue_degrees: f64,
    pub saturation: f64,
    pub lightness: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColorCurves {
    pub luma: Option<ToneCurve>,
    pub red: Option<ToneCurve>,
    pub green: Option<ToneCurve>,
    pub blue: Option<ToneCurve>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToneCurve {
    pub points: Vec<CurvePoint>,
    pub interpolation: ToneCurveInterpolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CurvePoint {
    pub input: f64,
    pub output: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToneCurveInterpolation {
    Natural,
    Monotonic,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColorWheel {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LiftGammaGain {
    pub lift: ColorWheel,
    pub gamma: ColorWheel,
    pub gain: ColorWheel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LutApplication {
    pub material_id: MaterialId,
    pub interpolation: LutInterpolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LutInterpolation {
    Nearest,
    Linear,
    Cosine,
    Cubic,
    Spline,
    Trilinear,
    Tetrahedral,
    Pyramid,
    Prism,
}
