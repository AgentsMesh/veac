/// Clip-level IR types with all time values resolved to seconds.
use std::path::PathBuf;

use super::IrAssetKind;

/// A fully resolved clip with all times in seconds.
#[derive(Debug, Clone)]
pub struct IrClip {
    pub asset_name: String,
    pub asset_path: PathBuf,
    pub asset_kind: IrAssetKind,
    pub from_sec: Option<f64>,
    pub to_sec: Option<f64>,
    pub duration_sec: Option<f64>,
    pub volume: Option<f64>,
    // P1: speed control
    pub speed: Option<f64>,
    // P1: audio fade
    pub fade_in_sec: Option<f64>,
    pub fade_out_sec: Option<f64>,
    // P2: color grading
    pub brightness: Option<f64>,
    pub contrast: Option<f64>,
    pub saturation: Option<f64>,
    /// Peak zoom level for zoompan animation (e.g. 2.0 = 2x zoom).
    /// Animates: normal → zoom in → hold → zoom out.
    pub zoom: Option<f64>,
    /// Whether the source asset contains an audio stream.
    /// When false, codegen will generate silence instead of referencing the input audio.
    pub has_audio: bool,
    // Video transform effects
    pub crop: Option<String>,
    pub blur: Option<f64>,
    pub opacity: Option<f64>,
    pub rotate: Option<f64>,
    pub flip: Option<String>,
    pub vignette: Option<f64>,
    pub grain: Option<f64>,
    pub sharpen: Option<f64>,
    // Pan offsets for zoompan focus point (-1.0 to 1.0)
    pub pan_x: Option<f64>,
    pub pan_y: Option<f64>,
    // Reverse playback
    pub reverse: Option<bool>,
    // Chroma key color (e.g. "green", "blue", "#00FF00")
    pub chromakey: Option<String>,
    // Audio normalization (loudnorm)
    pub normalize: Option<bool>,
    // Loop count: repeat clip N times
    pub loop_count: Option<u32>,
    // Video stabilization (deshake)
    pub stabilize: Option<bool>,
}

impl Default for IrClip {
    fn default() -> Self {
        Self {
            asset_name: String::new(),
            asset_path: PathBuf::new(),
            asset_kind: IrAssetKind::Video,
            from_sec: None,
            to_sec: None,
            duration_sec: None,
            volume: None,
            speed: None,
            fade_in_sec: None,
            fade_out_sec: None,
            brightness: None,
            contrast: None,
            saturation: None,
            zoom: None,
            has_audio: true,
            crop: None,
            blur: None,
            opacity: None,
            rotate: None,
            flip: None,
            vignette: None,
            grain: None,
            sharpen: None,
            pan_x: None,
            pan_y: None,
            reverse: None,
            chromakey: None,
            normalize: None,
            loop_count: None,
            stabilize: None,
        }
    }
}
