/// Media metadata obtained from probing source files via ffprobe.
///
/// Populated by `MediaResolver` during the resolve pass between semantic
/// analysis and codegen. When probing is skipped (`--no-probe`), fields
/// remain at their defaults and codegen falls back to estimation with
/// compiler warnings.
#[derive(Debug, Clone)]
pub struct MediaInfo {
    /// Total duration in seconds.
    pub duration_secs: f64,
    /// Whether the file contains a video stream.
    pub has_video: bool,
    /// Whether the file contains an audio stream.
    pub has_audio: bool,
    /// Video width in pixels (if has_video).
    pub width: Option<u32>,
    /// Video height in pixels (if has_video).
    pub height: Option<u32>,
    /// Audio sample rate in Hz (if has_audio).
    pub sample_rate: Option<u32>,
}
