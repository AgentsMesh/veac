use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Color, Length, Point, RationalTime, Rect, Vec2};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TemporalType {
    Boolean,
    Integer,
    Scalar,
    Time,
    Length,
    Angle,
    Vec2,
    Point,
    Rect,
    Color,
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TemporalValue {
    Boolean { value: bool },
    Integer { value: i64 },
    Scalar { value: f64 },
    Time { value: RationalTime },
    Length { value: Length },
    Angle { degrees: f64 },
    Vec2 { value: Vec2 },
    Point { value: Point },
    Rect { value: Rect },
    Color { value: Color },
    Text { value: String },
}

impl TemporalValue {
    pub const fn value_type(&self) -> TemporalType {
        match self {
            Self::Boolean { .. } => TemporalType::Boolean,
            Self::Integer { .. } => TemporalType::Integer,
            Self::Scalar { .. } => TemporalType::Scalar,
            Self::Time { .. } => TemporalType::Time,
            Self::Length { .. } => TemporalType::Length,
            Self::Angle { .. } => TemporalType::Angle,
            Self::Vec2 { .. } => TemporalType::Vec2,
            Self::Point { .. } => TemporalType::Point,
            Self::Rect { .. } => TemporalType::Rect,
            Self::Color { .. } => TemporalType::Color,
            Self::Text { .. } => TemporalType::Text,
        }
    }

    pub fn logical_bytes(&self) -> usize {
        match self {
            Self::Text { value } => 24usize.saturating_add(value.len()),
            Self::Point { .. } | Self::Rect { .. } => 32,
            Self::Vec2 { .. } | Self::Length { .. } | Self::Time { .. } => 16,
            Self::Color { .. } => 4,
            Self::Boolean { .. } => 1,
            Self::Integer { .. } | Self::Scalar { .. } | Self::Angle { .. } => 8,
        }
    }
}
