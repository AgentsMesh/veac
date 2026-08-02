use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Animatable, Vec2};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mask {
    pub shape: MaskShape,
    pub position: Animatable<Vec2>,
    pub scale: Animatable<Vec2>,
    pub rotation_degrees: Animatable<f64>,
    pub feather_pixels: Animatable<f64>,
    pub expansion_pixels: Animatable<f64>,
    pub invert: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MaskShape {
    Linear,
    Mirror,
    Circle,
    Rectangle,
    RoundedRectangle { radius: f64 },
    Ellipse,
    Polygon { points: Vec<Vec2> },
    Heart,
    Star,
    Path { points: Vec<Vec2> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrackMatteMode {
    Alpha,
    Luma,
}
