mod arithmetic;
mod visual;

pub use arithmetic::*;
pub use visual::*;

/// Long-form edits remain representable, while accidental multi-day timelines fail closed.
pub const MAX_TIMELINE_SECONDS: u64 = 86_400;
/// Final video frames emitted by one deliverable.
pub const MAX_VIDEO_FRAMES_PER_DELIVERABLE: u128 = 2_592_000;
/// Final frame count multiplied by output width and height.
pub const MAX_PIXEL_FRAMES_PER_DELIVERABLE: u128 = 4_000_000_000_000;
/// Sample values across all output channels, not merely sample frames.
pub const MAX_AUDIO_SAMPLES_PER_DELIVERABLE: u128 = 8_294_400_000;
/// FFmpeg's reverse filters buffer an entire source interval in memory.
pub const MAX_REVERSE_BUFFERED_SECONDS: u64 = 30;
/// About 512 MiB for a single 8-bit RGBA intermediate before backend overhead.
pub const MAX_VISUAL_INTERMEDIATE_PIXELS: u128 = 134_217_728;

pub const MAX_TOTAL_TRACKS: u64 = 512;
pub const MAX_TOTAL_CLIPS: u64 = 16_384;
pub const MAX_TOTAL_EFFECTS: u64 = 65_536;
pub const MAX_TOTAL_MASKS: u64 = 32_768;
pub const MAX_TOTAL_KEYFRAMES: u64 = 262_144;
pub const MAX_TOTAL_SOURCE_CURVE_SEGMENTS: u64 = 65_536;
pub const MAX_TOTAL_CAPTION_CUES: u64 = 10_000;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RenderStructureUsage {
    pub tracks: u64,
    pub clips: u64,
    pub effects: u64,
    pub masks: u64,
    pub keyframes: u64,
    pub source_curve_segments: u64,
    pub caption_cues: u64,
}

impl RenderStructureUsage {
    pub fn add(&mut self, other: Self) {
        self.tracks = self.tracks.saturating_add(other.tracks);
        self.clips = self.clips.saturating_add(other.clips);
        self.effects = self.effects.saturating_add(other.effects);
        self.masks = self.masks.saturating_add(other.masks);
        self.keyframes = self.keyframes.saturating_add(other.keyframes);
        self.source_curve_segments = self
            .source_curve_segments
            .saturating_add(other.source_curve_segments);
        self.caption_cues = self.caption_cues.saturating_add(other.caption_cues);
    }
}
