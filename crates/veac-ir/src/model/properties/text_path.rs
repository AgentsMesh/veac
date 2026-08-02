use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Length, Point};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextPath {
    pub points: Vec<Point>,
    pub start_offset: Length,
    pub reverse: bool,
    pub alignment: TextPathAlignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextPathAlignment {
    Start,
    Center,
    End,
}
