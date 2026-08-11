use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RangeExpectation {
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AlphaExpectation {
    pub transparent_below: u8,
    pub opaque_above: u8,
    pub mean: Option<RangeExpectation>,
    pub transparent_fraction: Option<RangeExpectation>,
    pub partial_fraction: Option<RangeExpectation>,
    pub opaque_fraction: Option<RangeExpectation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiffExpectation {
    pub rmse: Option<RangeExpectation>,
    pub mae: Option<RangeExpectation>,
    pub maximum_delta: Option<RangeExpectation>,
    pub changed_fraction: Option<RangeExpectation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BoundsExpectation {
    pub non_empty: bool,
    pub minimum_margin_pixels: Option<u32>,
    pub maximum_width_pixels: Option<u32>,
    pub maximum_height_pixels: Option<u32>,
}
