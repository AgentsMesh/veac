/// Intermediate Representation — fully validated, all references resolved.
mod clip;
mod overlay;
mod project;

pub use clip::*;
pub use overlay::*;
pub use project::*;

use std::path::PathBuf;

/// Top-level validated IR produced by semantic analysis.
#[derive(Debug, Clone)]
pub struct IrProgram {
    pub project: IrProject,
    pub assets: Vec<IrAsset>,
    pub timeline: IrTimeline,
    pub outputs: Vec<IrOutputConfig>,
}

#[derive(Debug, Clone)]
pub struct IrAsset {
    pub name: String,
    pub kind: IrAssetKind,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IrAssetKind {
    Video,
    Audio,
    Image,
}

#[derive(Debug, Clone)]
pub struct IrTimeline {
    pub name: String,
    pub tracks: Vec<IrTrack>,
}

#[derive(Debug, Clone)]
pub struct IrTrack {
    pub kind: IrTrackKind,
    pub items: Vec<IrTrackItem>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IrTrackKind {
    Video,
    Audio,
    Text,
    Overlay,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum IrTrackItem {
    Clip(IrClip),
    TextOverlay(IrTextOverlay),
    Transition(IrTransition),
    ImageOverlay(IrImageOverlay),
    Gap(IrGap),
    Freeze(IrFreeze),
    Pip(IrPip),
    Subtitle(IrSubtitle),
}

/// A silent black gap in the timeline.
#[derive(Debug, Clone)]
pub struct IrGap {
    pub duration_sec: f64,
}

/// A freeze frame from a clip asset.
#[derive(Debug, Clone)]
pub struct IrFreeze {
    pub asset_name: String,
    pub asset_path: PathBuf,
    pub at_sec: f64,
    pub duration_sec: f64,
}

/// Picture-in-picture overlay.
#[derive(Debug, Clone)]
pub struct IrPip {
    pub asset_name: String,
    pub asset_path: PathBuf,
    pub from_sec: Option<f64>,
    pub to_sec: Option<f64>,
    pub at_sec: f64,
    pub duration_sec: f64,
    pub position: Position,
    pub scale: f64,
}

/// Subtitle (.srt) import.
#[derive(Debug, Clone)]
pub struct IrSubtitle {
    pub path: PathBuf,
}

/// Output configuration for multi-output support.
#[derive(Debug, Clone)]
pub struct IrOutputConfig {
    pub path: PathBuf,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<OutputFormat>,
    pub codec: Option<Codec>,
    pub quality: Option<Quality>,
}

/// How the video content is fitted into the output frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FitMode {
    Fill,
    Letterbox,
    Crop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
}

impl Position {
    /// Convert to FFmpeg drawtext x,y expressions.
    pub fn to_ffmpeg_xy(&self) -> (&str, &str) {
        match self {
            Position::Center => ("(w-text_w)/2", "(h-text_h)/2"),
            Position::TopLeft => ("10", "10"),
            Position::TopRight => ("w-text_w-10", "10"),
            Position::BottomLeft => ("10", "h-text_h-10"),
            Position::BottomRight => ("w-text_w-10", "h-text_h-10"),
            Position::Top => ("(w-text_w)/2", "10"),
            Position::Bottom => ("(w-text_w)/2", "h-text_h-10"),
            Position::Left => ("10", "(h-text_h)/2"),
            Position::Right => ("w-text_w-10", "(h-text_h)/2"),
        }
    }

    /// Convert to FFmpeg overlay x,y expressions for image overlay.
    pub fn to_overlay_xy(&self) -> (&str, &str) {
        match self {
            Position::Center => ("(W-w)/2", "(H-h)/2"),
            Position::TopLeft => ("10", "10"),
            Position::TopRight => ("W-w-10", "10"),
            Position::BottomLeft => ("10", "H-h-10"),
            Position::BottomRight => ("W-w-10", "H-h-10"),
            Position::Top => ("(W-w)/2", "10"),
            Position::Bottom => ("(W-w)/2", "H-h-10"),
            Position::Left => ("10", "(H-h)/2"),
            Position::Right => ("W-w-10", "(H-h)/2"),
        }
    }
}
