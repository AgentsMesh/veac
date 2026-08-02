use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{EffectId, ParameterValue, TimeRange};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedEffect {
    pub id: EffectId,
    pub effect_type: String,
    pub active_range: TimeRange,
    /// Parameters are key-sorted; effect instances retain canonical vector order on the clip.
    pub parameters: BTreeMap<String, ParameterValue>,
}
