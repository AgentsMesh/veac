use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Animatable;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Length {
    pub value: f64,
    pub unit: LengthUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LengthUnit {
    Pixels,
    Normalized,
    Percent,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: Length,
    pub y: Length,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Transform2D {
    pub position: Animatable<Point>,
    pub scale: Animatable<Vec2>,
    /// Unitless x/y shear factors in the closed range [-2, 2].
    pub shear: Vec2,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub rotation_degrees: Animatable<f64>,
    pub anchor: Vec2,
    /// A normalized viewport. Animated viewports retain the first keyframe's
    /// output extent while the sampled rectangle moves and resizes.
    pub crop: Option<Animatable<Rect>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Placement {
    Anchor { anchor: Anchor, inset: Vec2 },
    Absolute { position: Point },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    Center,
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub width: Length,
    pub height: Length,
    pub fit: FitMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FitMode {
    Fill,
    Contain,
    Cover,
}
