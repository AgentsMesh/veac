use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MEDIA_PROBE_SCHEMA_VERSION: u32 = 3;
use serde_json::Value;

use crate::{MaterialId, Rational, RationalTime};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub id: MaterialId,
    pub kind: MaterialKind,
    pub source: MaterialSource,
    pub identity: Option<MediaIdentity>,
    pub stream_intent: StreamIntent,
    /// Probe output is a reproducibility fact, not part of semantic content hashing.
    pub probe: Option<MediaProbeSnapshot>,
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MaterialKind {
    Video,
    Audio,
    Image,
    Font,
    Lut1d,
    Lut3d,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MaterialSource {
    File { uri: String },
    Remote { uri: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MediaIdentity {
    pub algorithm: HashAlgorithm,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MediaProbeSnapshot {
    pub schema_version: u32,
    pub engine: String,
    pub selection_policy: String,
    pub container_format: String,
    pub container_brand: Option<String>,
    pub observed_identity: MediaIdentity,
    pub container_duration: Option<RationalTime>,
    pub streams: Vec<ProbedStream>,
    /// Both indexes are retained because FFmpeg's `N:v` and global stream indexes are different
    /// namespaces. Attached pictures are never valid selected video streams.
    pub selected_video_stream: Option<StreamSelection>,
    pub selected_audio_stream: Option<StreamSelection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StreamSelection {
    pub global_index: u32,
    pub type_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StreamIntent {
    pub video: StreamChoice,
    pub audio: StreamChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum StreamChoice {
    Auto,
    Disabled,
    GlobalIndex { global_index: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProbedStream {
    pub global_index: u32,
    /// Ordinal among all streams of this media type, ordered by global index. Attached pictures
    /// participate in this ordinal; it is not FFmpeg's `V` playable-video namespace.
    pub type_index: u32,
    pub media_type: ProbedStreamType,
    pub codec: String,
    /// Seconds per stream timestamp tick, as reported by the pinned probe engine.
    pub time_base: Option<Rational>,
    pub start_time: Option<RationalTime>,
    pub duration: Option<RationalTime>,
    pub disposition: StreamDisposition,
    pub video: Option<VideoStreamInfo>,
    pub audio: Option<AudioStreamInfo>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProbedStreamType {
    Video,
    Audio,
    Subtitle,
    Data,
    Attachment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StreamDisposition {
    pub default: bool,
    pub attached_picture: bool,
    pub timed_thumbnail: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VideoStreamInfo {
    pub width: u32,
    pub height: u32,
    /// Canonical average frame rate, falling back to the declared frame rate when necessary.
    pub frame_rate: Option<Rational>,
    pub pixel_format: String,
    pub profile: Option<String>,
    pub level: Option<i32>,
    pub sample_aspect_ratio: Rational,
    pub rotation_degrees: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudioStreamInfo {
    pub sample_rate: u32,
    pub channels: u8,
    pub channel_layout: String,
}
