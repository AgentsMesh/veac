use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    MulticamAngleId, MulticamGroupId, MulticamSwitch, MulticamSync, RationalTime, StreamSelection,
};

use super::PlanInputId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedMulticamSource {
    pub group_id: MulticamGroupId,
    pub sync: MulticamSync,
    pub angles: Vec<ResolvedMulticamAngle>,
    pub switches: Vec<MulticamSwitch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedMulticamAngle {
    pub id: MulticamAngleId,
    pub input_id: PlanInputId,
    pub video_stream: StreamSelection,
    pub audio_stream: Option<StreamSelection>,
    pub source_offset: RationalTime,
}
