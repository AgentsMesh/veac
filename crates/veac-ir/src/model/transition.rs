use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::RationalTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub kind: TransitionKind,
    pub duration: RationalTime,
    pub alignment: TransitionAlignment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransitionKind {
    Dissolve,
    Fade {
        color: FadeColor,
    },
    Wipe {
        direction: CardinalDirection,
        angle_degrees: f64,
        softness: f64,
    },
    Slide {
        direction: CardinalDirection,
        amount: f64,
    },
    Zoom {
        direction: ZoomDirection,
        amount: f64,
    },
    Circle {
        direction: CircleDirection,
        softness: f64,
    },
    Pixelize {
        amount: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FadeColor {
    Transparent,
    Black,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CardinalDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ZoomDirection {
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CircleDirection {
    Open,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransitionAlignment {
    Centered,
}

pub fn transition_parameters_valid(kind: &TransitionKind) -> bool {
    match kind {
        TransitionKind::Dissolve | TransitionKind::Fade { .. } => true,
        TransitionKind::Wipe {
            angle_degrees,
            softness,
            ..
        } => angle_degrees.is_finite() && unit(*softness),
        TransitionKind::Slide { amount, .. } | TransitionKind::Zoom { amount, .. } => {
            amount.is_finite() && (0.1..=4.0).contains(amount)
        }
        TransitionKind::Circle { softness, .. } => unit(*softness),
        TransitionKind::Pixelize { amount } => unit(*amount) && *amount > 0.0,
    }
}

fn unit(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
