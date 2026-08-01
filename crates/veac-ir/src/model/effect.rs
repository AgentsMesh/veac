use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Animatable, Color, EffectId, TimeRange};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectInstance {
    pub id: EffectId,
    pub effect_type: String,
    pub enabled: bool,
    pub enable_range: Option<TimeRange>,
    pub parameters: BTreeMap<String, ParameterValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ParameterValue {
    Number { value: f64 },
    Boolean { value: bool },
    Color { value: Color },
    NumberCurve { value: Animatable<f64> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParameterValueKind {
    Number,
    NumberCurve,
    Boolean,
    Color,
}

impl ParameterValueKind {
    pub const ALL: [Self; 4] = [Self::Number, Self::NumberCurve, Self::Boolean, Self::Color];

    pub const fn schema_name(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::NumberCurve => "number_curve",
            Self::Boolean => "boolean",
            Self::Color => "color",
        }
    }
}

impl ParameterValue {
    pub const fn kind(&self) -> ParameterValueKind {
        match self {
            Self::Number { .. } => ParameterValueKind::Number,
            Self::NumberCurve { .. } => ParameterValueKind::NumberCurve,
            Self::Boolean { .. } => ParameterValueKind::Boolean,
            Self::Color { .. } => ParameterValueKind::Color,
        }
    }
}
