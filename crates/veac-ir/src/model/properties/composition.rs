use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Animatable, ColorPipeline, Frame, Mask, Placement, Transform2D, Vec2};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VisualProperties {
    pub placement: Placement,
    pub frame: Option<Frame>,
    pub transform: Transform2D,
    pub opacity: Animatable<f64>,
    pub compositing: Compositing,
    pub masks: Vec<Mask>,
    pub card: Option<CardStyle>,
    pub color_pipeline: Option<ColorPipeline>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Compositing {
    pub z_index: i32,
    pub blend_mode: BlendMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CardStyle {
    pub corner_radius_pixels: f64,
    pub shadow: Option<Shadow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Shadow {
    pub blur_pixels: f64,
    pub opacity: f64,
    pub offset: Vec2,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
