use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    AudioProperties, ItemId, Material, MaterialId, RationalTime, SequenceId, TimeRange, TrackId,
};

use crate::StemKind;

use super::ApplicationHeader;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MaterialInsertion {
    pub material: Material,
    pub before_id: Option<MaterialId>,
    pub after_id: Option<MaterialId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudioClipInsertion {
    pub material: MaterialInsertion,
    pub sequence_id: SequenceId,
    pub track_id: TrackId,
    pub clip_id: ItemId,
    pub record_range: TimeRange,
    pub source_start: RationalTime,
    pub audio: AudioProperties,
    pub before_id: Option<ItemId>,
    pub after_id: Option<ItemId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GeneratedAudioApplication {
    pub header: ApplicationHeader,
    pub output: AudioClipInsertion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DubbingApplication {
    pub header: ApplicationHeader,
    pub source_clip_id: ItemId,
    pub output: AudioClipInsertion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MediaReplacementApplication {
    pub header: ApplicationHeader,
    pub material: MaterialInsertion,
    pub clip_id: ItemId,
    pub source_start: RationalTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StemBinding {
    pub kind: StemKind,
    pub label: String,
    pub output: AudioClipInsertion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SeparationApplication {
    pub header: ApplicationHeader,
    pub source_clip_id: ItemId,
    pub bindings: Vec<StemBinding>,
}
