use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ItemId, SequenceId};

use super::{
    TemporalBindingId, TemporalClock, TemporalInputId, TemporalParameterId, TemporalProgramId,
    TemporalProvenanceId, TemporalType, TemporalValue,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TemporalClockOwner {
    Sequence { sequence_id: SequenceId },
    Item { item_id: ItemId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalClockBinding {
    pub input_id: TemporalInputId,
    pub clock: TemporalClock,
    pub owner: TemporalClockOwner,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalParameterBinding {
    pub input_id: TemporalInputId,
    pub parameter_id: TemporalParameterId,
    pub value: TemporalValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalBinding {
    pub id: TemporalBindingId,
    pub program_id: TemporalProgramId,
    pub result_type: TemporalType,
    pub clocks: Vec<TemporalClockBinding>,
    pub parameters: Vec<TemporalParameterBinding>,
    pub provenance_id: TemporalProvenanceId,
}
