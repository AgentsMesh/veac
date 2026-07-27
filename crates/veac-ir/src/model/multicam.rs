use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{MaterialId, MulticamAngleId, MulticamGroupId, RationalTime, TimeRange};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MulticamGroup {
    pub id: MulticamGroupId,
    pub sync: MulticamSync,
    pub angles: Vec<MulticamAngle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MulticamSync {
    pub basis: MulticamSyncBasis,
    pub reference_angle_id: MulticamAngleId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MulticamSyncBasis {
    Timecode,
    Audio,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MulticamAngle {
    pub id: MulticamAngleId,
    pub material_id: MaterialId,
    /// Source time corresponding to group-local time zero after synchronization.
    pub source_offset: RationalTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MulticamSwitch {
    pub angle_id: MulticamAngleId,
    /// Clip-local range; authored segments must form one contiguous partition.
    pub range: TimeRange,
}
