use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Color, Rect, Vec2};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Generator {
    Solid { color: Color },
    Gradient { gradient: Gradient },
    Shape { shape: VectorShape },
    Transparent,
    Silence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Gradient {
    Linear {
        start: Vec2,
        end: Vec2,
        stops: Vec<GradientStop>,
    },
    Radial {
        center: Vec2,
        radius: f64,
        stops: Vec<GradientStop>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GradientStop {
    pub offset: f64,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorShape {
    pub geometry: VectorGeometry,
    pub fill: Option<Paint>,
    pub stroke: Option<VectorStroke>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Paint {
    Solid { color: Color },
    Gradient { gradient: Gradient },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorStroke {
    pub paint: Paint,
    pub width_pixels: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorGeometry {
    Rectangle { bounds: Rect },
    Ellipse { bounds: Rect },
    RoundedRectangle { bounds: Rect, radius: f64 },
    Polygon { points: Vec<Vec2> },
    Path { commands: Vec<PathCommand> },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum PathCommand {
    MoveTo { point: Vec2 },
    LineTo { point: Vec2 },
    Close,
}
