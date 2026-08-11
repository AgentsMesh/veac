mod arithmetic;
mod visual;

pub use arithmetic::*;
pub use visual::*;

use crate::{
    MAX_TEMPORAL_BINDINGS, MAX_TEMPORAL_NODES, MAX_TEMPORAL_PROGRAMS, MAX_TEMPORAL_PROVENANCE,
};

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
/// Maximum conservative decoded-buffer sum for all reverse instances emitted by one command.
pub const MAX_REVERSE_BUFFERED_BYTES: u128 = 1024 * 1024 * 1024;
/// Eight bytes cover planar 16-bit RGBA; another 2x covers planes, alignment, and filter ownership.
pub const REVERSE_DECODED_BYTES_PER_PIXEL: u128 = 16;
/// Conservative per-frame AVFrame, buffer-reference, and allocator overhead.
pub const REVERSE_FRAME_OVERHEAD_BYTES: u128 = 4 * 1024;
/// Eight bytes cover decoded f64; another 2x covers planes, alignment, and filter ownership.
pub const REVERSE_DECODED_BYTES_PER_CHANNEL_SAMPLE: u128 = 16;
/// About 512 MiB for a single 8-bit RGBA intermediate before backend overhead.
pub const MAX_VISUAL_INTERMEDIATE_PIXELS: u128 = 134_217_728;

pub const MAX_TOTAL_TRACKS: u64 = 512;
pub const MAX_TOTAL_CLIPS: u64 = 16_384;
pub const MAX_TOTAL_EFFECTS: u64 = 65_536;
pub const MAX_TOTAL_MASKS: u64 = 32_768;
pub const MAX_TOTAL_KEYFRAMES: u64 = 262_144;
pub const MAX_TOTAL_SOURCE_CURVE_SEGMENTS: u64 = 65_536;
pub const MAX_TOTAL_CAPTION_CUES: u64 = 10_000;
pub const MAX_TOTAL_TEMPORAL_PROGRAMS: u64 = MAX_TEMPORAL_PROGRAMS as u64;
pub const MAX_TOTAL_TEMPORAL_BINDINGS: u64 = MAX_TEMPORAL_BINDINGS as u64;
pub const MAX_TOTAL_TEMPORAL_NODES: u64 = MAX_TEMPORAL_NODES as u64;
pub const MAX_TOTAL_TEMPORAL_PROVENANCE: u64 = MAX_TEMPORAL_PROVENANCE as u64;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RenderStructureUsage {
    pub tracks: u64,
    pub clips: u64,
    pub effects: u64,
    pub masks: u64,
    pub keyframes: u64,
    pub source_curve_segments: u64,
    pub caption_cues: u64,
    pub temporal_programs: u64,
    pub temporal_bindings: u64,
    pub temporal_nodes: u64,
    pub temporal_provenance: u64,
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
        self.temporal_programs = self
            .temporal_programs
            .saturating_add(other.temporal_programs);
        self.temporal_bindings = self
            .temporal_bindings
            .saturating_add(other.temporal_bindings);
        self.temporal_nodes = self.temporal_nodes.saturating_add(other.temporal_nodes);
        self.temporal_provenance = self
            .temporal_provenance
            .saturating_add(other.temporal_provenance);
    }
}
