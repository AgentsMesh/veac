use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    AudioStreamInfo, MaterialId, MaterialKind, MediaIdentity, RationalTime, StreamDisposition,
    StreamSelection, VideoStreamInfo,
};

use super::PlanInputId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedInput {
    pub id: PlanInputId,
    pub material_id: Option<MaterialId>,
    pub kind: ResolvedInputKind,
    pub canonical_uri: String,
    pub observed_identity: MediaIdentity,
    pub probe: Option<ResolvedMediaProbe>,
    pub video: Option<ResolvedVideoStream>,
    pub audio: Option<ResolvedAudioStream>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolvedInputKind {
    Media {
        material_kind: MaterialKind,
    },
    Font {
        family: Option<String>,
        postscript_name: Option<String>,
        face_index: u32,
    },
    Resource {
        material_kind: MaterialKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedMediaProbe {
    pub schema_version: u32,
    pub engine: String,
    pub selection_policy: String,
    pub container_format: String,
    pub container_duration: Option<RationalTime>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedVideoStream {
    pub selection: StreamSelection,
    pub codec: String,
    pub start_time: Option<RationalTime>,
    pub duration: Option<RationalTime>,
    pub disposition: StreamDisposition,
    pub info: VideoStreamInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedAudioStream {
    pub selection: StreamSelection,
    pub codec: String,
    pub start_time: Option<RationalTime>,
    pub duration: Option<RationalTime>,
    pub disposition: StreamDisposition,
    pub info: AudioStreamInfo,
}
