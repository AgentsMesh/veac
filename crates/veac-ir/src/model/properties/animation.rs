use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{KeyframeId, RationalTime};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Animatable<T> {
    Constant { value: T },
    Keyframes { keyframes: Vec<Keyframe<T>> },
}

impl<T> Animatable<T> {
    pub fn constant(value: T) -> Self {
        Self::Constant { value }
    }

    pub fn keyframes(&self) -> Option<&[Keyframe<T>]> {
        match self {
            Self::Constant { .. } => None,
            Self::Keyframes { keyframes } => Some(keyframes),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Keyframe<T> {
    pub id: KeyframeId,
    pub time: RationalTime,
    pub value: T,
    pub interpolation: Interpolation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Interpolation {
    Hold,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Spring {
        frequency: f64,
        decay: f64,
        initial_velocity: f64,
    },
    CubicBezier {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringCoefficients {
    pub angular_frequency: f64,
    pub equilibrium: f64,
    pub sine: f64,
}
