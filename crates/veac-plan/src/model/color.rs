use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    BasicColorAdjustment, ColorCurves, ColorSpace, HslAdjustment, LiftGammaGain, LutInterpolation,
    RgbMatrixAdjustment,
};

use super::PlanInputId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedColorPipeline {
    pub input: ColorSpace,
    pub working: ColorSpace,
    pub output: ColorSpace,
    pub stages: Vec<ResolvedColorStage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolvedColorStage {
    Basic { adjustment: BasicColorAdjustment },
    Matrix { adjustment: RgbMatrixAdjustment },
    Hsl { adjustment: HslAdjustment },
    Curves { curves: ColorCurves },
    Wheels { wheels: LiftGammaGain },
    Lut { application: ResolvedLut },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedLut {
    pub input_id: PlanInputId,
    pub kind: ResolvedLutKind,
    pub interpolation: LutInterpolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResolvedLutKind {
    OneDimensional,
    ThreeDimensional,
}
