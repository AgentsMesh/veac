use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Animatable, Color, Length, LengthUnit, Point, RationalTime, Vec2};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextAnimation {
    pub granularity: TextGranularity,
    pub transform: TextUnitTransform,
    /// Visible fraction of units in logical reading order.
    pub reveal: Animatable<f64>,
    /// Optional active fill moving through logical reading order.
    #[serde(deserialize_with = "explicit_option")]
    #[schemars(with = "Option<TextHighlightAnimation>", required)]
    pub highlight: Option<TextHighlightAnimation>,
    /// Per-unit opacity curve. Unit N evaluates it N * stagger later.
    pub opacity: Animatable<f64>,
    pub stagger: RationalTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextHighlightAnimation {
    pub fill: Color,
    pub progress: Animatable<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextUnitTransform {
    pub position_offset: Animatable<Point>,
    pub scale: Animatable<Vec2>,
    pub rotation_degrees: Animatable<f64>,
}

impl Default for TextUnitTransform {
    fn default() -> Self {
        Self {
            position_offset: Animatable::constant(Point {
                x: Length {
                    value: 0.0,
                    unit: LengthUnit::Pixels,
                },
                y: Length {
                    value: 0.0,
                    unit: LengthUnit::Pixels,
                },
            }),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            rotation_degrees: Animatable::constant(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextGranularity {
    Whole,
    Line,
    Word,
    Grapheme,
}

fn explicit_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}
