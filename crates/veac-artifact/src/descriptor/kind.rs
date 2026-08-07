use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    ProxyVideo,
    ProxyAudio,
    Waveform,
    Thumbnail,
    Speech,
    Translation,
    MotionTrack,
    Matte,
    OpticalFlow,
    Analysis,
    SourceSegment,
    RenderSegment,
    CaptionSidecar,
    AudioStem,
    AudioFile,
    AnimatedImage,
    StillImage,
    AdaptivePackage,
    VideoMaster,
    ImageSequenceFrame,
    VideoWaveform,
    Vectorscope,
    Histogram,
    RenderCheckpoint,
}
