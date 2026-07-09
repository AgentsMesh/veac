/// Intermediate Representation — fully validated, all references resolved.
mod clip;
mod media_info;
mod overlay;
mod project;

pub use clip::*;
pub use media_info::*;
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
    /// Media metadata from probing. Populated by `MediaResolver`.
    /// `None` when probing is skipped or the file cannot be probed.
    pub media_info: Option<MediaInfo>,
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
    /// Entrance zoom: seconds to animate from full-frame → `scale` (0 = no animation, snap to scale).
    pub zoom_in_sec: f64,
    /// Exit zoom: seconds to animate from `scale` → full-frame at the end (0 = none).
    pub zoom_out_sec: f64,
    /// Entrance alpha fade: seconds to ramp opacity 0 → 1 at the start (0 = none). Lets a
    /// full-frame B-roll pip dissolve in over the base track instead of hard-cutting.
    pub fade_in_sec: f64,
    /// Exit alpha fade: seconds to ramp opacity 1 → 0 at the end (0 = none).
    pub fade_out_sec: f64,
    /// Horizontal inset in px from the anchored edge at the pip's resting (corner) size. For a
    /// zooming pip the inset interpolates with the zoom — 0 at full frame (exact fill, no ghost
    /// against the base track), reaching `margin_x` only at corner size. Keeps the overlay off the
    /// screen edge (e.g. clearing a vertical player's side UI). Ignored for center anchoring.
    pub margin_x: f64,
    /// Vertical inset in px from the anchored edge at the pip's resting (corner) size. Same
    /// zoom-interpolated behavior as `margin_x`; used to lift a bottom-anchored cam above the
    /// caption/subtitle safe zone.
    pub margin_y: f64,
    /// Explicit pip width in px, overriding `scale * output_width`. `0` = derive from `scale`.
    /// Needed for a square overlay on a non-square canvas (a portrait project's `scale` yields a
    /// rectangle, which would squash a circular bubble into an ellipse).
    pub width: f64,
    /// Explicit pip height in px, overriding `scale * output_height`. `0` = derive from `scale`.
    pub height: f64,
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

    /// Margin-less overlay anchor for an animated (zooming) pip, so a full-frame `w==W` sits at
    /// x=0 (exact fill) and shrinks toward this corner. Used with `eval=frame` overlay.
    pub fn to_overlay_anchor(&self) -> (&str, &str) {
        match self {
            Position::Center => ("(W-w)/2", "(H-h)/2"),
            Position::TopLeft => ("0", "0"),
            Position::TopRight => ("W-w", "0"),
            Position::BottomLeft => ("0", "H-h"),
            Position::BottomRight => ("W-w", "H-h"),
            Position::Top => ("(W-w)/2", "0"),
            Position::Bottom => ("(W-w)/2", "H-h"),
            Position::Left => ("0", "(H-h)/2"),
            Position::Right => ("W-w", "(H-h)/2"),
        }
    }
}
