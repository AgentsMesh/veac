use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Animatable, Color, EffectId, RationalTime, TimeRange, Vec2};

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
    Integer { value: i64 },
    Boolean { value: bool },
    Text { value: String },
    Color { value: Color },
    Vec2 { value: Vec2 },
    Time { value: RationalTime },
    NumberCurve { value: Animatable<f64> },
}
