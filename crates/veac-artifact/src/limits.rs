/// Maximum on-disk payload accepted by the artifact store or media workflow.
pub const MAX_ARTIFACT_PAYLOAD_BYTES: u64 = 16 * 1024 * 1024 * 1024;
pub const MAX_VERIFIED_SOURCE_BYTES: u64 = 64 * 1024 * 1024 * 1024;
/// Maximum aggregate filesystem output produced by one render task.
pub const MAX_RENDER_TASK_OUTPUT_BYTES: u64 = 64 * 1024 * 1024 * 1024;
/// Whole-payload APIs are for small artifacts; large media must use `ArtifactStore::open`.
pub const MAX_IN_MEMORY_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_ANALYSIS_PAYLOAD_BYTES: u64 = MAX_IN_MEMORY_ARTIFACT_BYTES;
/// Descriptor and record JSON are control-plane data, not media payloads.
pub const MAX_ARTIFACT_METADATA_BYTES: u64 = 1024 * 1024;
pub const MAX_ARTIFACT_JSON_DEPTH: usize = 64;
pub const MAX_ARTIFACT_JSON_NODES: usize = 65_536;
pub const MAX_ARTIFACT_JSON_STRING_BYTES: usize = 512 * 1024;
pub const MAX_ARTIFACT_DEPENDENCIES: usize = 4_096;

pub const MAX_ARTIFACT_DIMENSION: u32 = veac_ir::MAX_DIMENSION;
pub const MAX_ARTIFACT_FRAME_PIXELS: u64 = veac_ir::MAX_FRAME_PIXELS;
pub const MAX_ARTIFACT_FRAME_RATE: u32 = veac_ir::MAX_FRAME_RATE;
pub const MAX_ARTIFACT_SAMPLE_RATE: u32 = veac_ir::PCM_MAX_SAMPLE_RATE;
pub const MAX_ARTIFACT_DURATION_SECONDS: u64 = veac_ir::MAX_TIMELINE_SECONDS;
pub const MAX_MEDIA_DERIVATION_CPU_SECONDS: u64 = 6 * 60 * 60;
pub const MAX_MEDIA_DERIVATION_WALL_SECONDS: u64 = 6 * 60 * 60;
pub const MAX_ARTIFACT_VIDEO_FRAMES: u128 = veac_ir::MAX_VIDEO_FRAMES_PER_DELIVERABLE;
pub const MAX_ARTIFACT_PIXEL_FRAMES: u128 = veac_ir::MAX_PIXEL_FRAMES_PER_DELIVERABLE;
pub const MAX_ARTIFACT_AUDIO_CHANNEL_SAMPLES: u128 = veac_ir::MAX_AUDIO_SAMPLES_PER_DELIVERABLE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaArtifactLimits {
    pub max_dimension: u32,
    pub max_frame_pixels: u64,
    pub max_frame_rate: u32,
    pub max_sample_rate: u32,
    pub max_duration_seconds: u64,
    pub max_derivation_cpu_seconds: u64,
    pub max_derivation_wall_seconds: u64,
    pub max_video_frames: u128,
    pub max_pixel_frames: u128,
    pub max_audio_channel_samples: u128,
    pub max_payload_bytes: u64,
    pub max_source_bytes: u64,
}

impl Default for MediaArtifactLimits {
    fn default() -> Self {
        Self {
            max_dimension: MAX_ARTIFACT_DIMENSION,
            max_frame_pixels: MAX_ARTIFACT_FRAME_PIXELS,
            max_frame_rate: MAX_ARTIFACT_FRAME_RATE,
            max_sample_rate: MAX_ARTIFACT_SAMPLE_RATE,
            max_duration_seconds: MAX_ARTIFACT_DURATION_SECONDS,
            max_derivation_cpu_seconds: MAX_MEDIA_DERIVATION_CPU_SECONDS,
            max_derivation_wall_seconds: MAX_MEDIA_DERIVATION_WALL_SECONDS,
            max_video_frames: MAX_ARTIFACT_VIDEO_FRAMES,
            max_pixel_frames: MAX_ARTIFACT_PIXEL_FRAMES,
            max_audio_channel_samples: MAX_ARTIFACT_AUDIO_CHANNEL_SAMPLES,
            max_payload_bytes: MAX_ARTIFACT_PAYLOAD_BYTES,
            max_source_bytes: MAX_VERIFIED_SOURCE_BYTES,
        }
    }
}

impl MediaArtifactLimits {
    pub(crate) fn is_valid(self) -> bool {
        let hard = Self::default();
        self.max_dimension > 0
            && self.max_dimension <= hard.max_dimension
            && self.max_frame_pixels > 0
            && self.max_frame_pixels <= hard.max_frame_pixels
            && self.max_frame_rate > 0
            && self.max_frame_rate <= hard.max_frame_rate
            && self.max_sample_rate > 0
            && self.max_sample_rate <= hard.max_sample_rate
            && self.max_duration_seconds > 0
            && self.max_duration_seconds <= hard.max_duration_seconds
            && self.max_derivation_cpu_seconds > 0
            && self.max_derivation_cpu_seconds <= hard.max_derivation_cpu_seconds
            && self.max_derivation_wall_seconds > 0
            && self.max_derivation_wall_seconds <= hard.max_derivation_wall_seconds
            && self.max_video_frames > 0
            && self.max_video_frames <= hard.max_video_frames
            && self.max_pixel_frames > 0
            && self.max_pixel_frames <= hard.max_pixel_frames
            && self.max_audio_channel_samples > 0
            && self.max_audio_channel_samples <= hard.max_audio_channel_samples
            && self.max_payload_bytes > 0
            && self.max_payload_bytes <= hard.max_payload_bytes
            && self.max_source_bytes > 0
            && self.max_source_bytes <= hard.max_source_bytes
    }
}
