use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{Effect, EffectId, TimeRange};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedEffect {
    pub id: EffectId,
    pub active_range: TimeRange,
    pub effect: Effect,
}
