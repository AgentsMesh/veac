/// Overlay and transition IR types.
use std::path::PathBuf;

use super::Position;

/// A fully resolved text overlay with all times in seconds.
#[derive(Debug, Clone)]
pub struct IrTextOverlay {
    pub content: String,
    pub at_sec: f64,
    pub duration_sec: f64,
    pub font: String,
    pub size: u32,
    pub color: String,
    pub position: Position,
    /// Optional text fade-in duration in seconds.
    pub fade_in_sec: Option<f64>,
    /// Optional text fade-out duration in seconds.
    pub fade_out_sec: Option<f64>,
    /// Resolved absolute path to the font file. Populated by `MediaResolver`.
    /// When `None`, codegen falls back to fontconfig-based `font=` parameter.
    pub resolved_font_path: Option<String>,
    /// Background box color with opacity (e.g. "black@0.5").
    /// When `Some`, renders a semi-transparent box behind the text.
    /// Default: `Some("black@0.5")` for readability.
    pub background: Option<String>,
    /// Background box padding in pixels. Default: 12.
    pub background_padding: Option<u32>,
}

/// A transition between two adjacent clips.
#[derive(Debug, Clone)]
pub struct IrTransition {
    pub kind: TransitionKind,
    pub duration_sec: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionKind {
    Fade,
    FadeBlack,
    FadeWhite,
    Dissolve,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    ZoomIn,
    SmoothLeft,
    SmoothRight,
    SmoothUp,
    SmoothDown,
    SqueezeH,
    SqueezeV,
    CircleCrop,
    Pixelize,
}

impl TransitionKind {
    /// Convert to the FFmpeg xfade transition name.
    pub fn to_ffmpeg(&self) -> &str {
        match self {
            TransitionKind::Fade => "fade",
            TransitionKind::FadeBlack => "fadeblack",
            TransitionKind::FadeWhite => "fadewhite",
            TransitionKind::Dissolve => "dissolve",
            TransitionKind::WipeLeft => "wipeleft",
            TransitionKind::WipeRight => "wiperight",
            TransitionKind::WipeUp => "wipeup",
            TransitionKind::WipeDown => "wipedown",
            TransitionKind::SlideLeft => "slideleft",
            TransitionKind::SlideRight => "slideright",
            TransitionKind::SlideUp => "slideup",
            TransitionKind::SlideDown => "slidedown",
            TransitionKind::ZoomIn => "zoomin",
            TransitionKind::SmoothLeft => "smoothleft",
            TransitionKind::SmoothRight => "smoothright",
            TransitionKind::SmoothUp => "smoothup",
            TransitionKind::SmoothDown => "smoothdown",
            TransitionKind::SqueezeH => "squeezeh",
            TransitionKind::SqueezeV => "squeezev",
            TransitionKind::CircleCrop => "circlecrop",
            TransitionKind::Pixelize => "pixelize",
        }
    }

    /// Parse from a user-facing string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "fade" => Some(TransitionKind::Fade),
            "fadeblack" | "fade-black" => Some(TransitionKind::FadeBlack),
            "fadewhite" | "fade-white" => Some(TransitionKind::FadeWhite),
            "dissolve" => Some(TransitionKind::Dissolve),
            "wipe-left" | "wipeleft" => Some(TransitionKind::WipeLeft),
            "wipe-right" | "wiperight" => Some(TransitionKind::WipeRight),
            "wipe-up" | "wipeup" => Some(TransitionKind::WipeUp),
            "wipe-down" | "wipedown" => Some(TransitionKind::WipeDown),
            "slide-left" | "slideleft" => Some(TransitionKind::SlideLeft),
            "slide-right" | "slideright" => Some(TransitionKind::SlideRight),
            "slide-up" | "slideup" => Some(TransitionKind::SlideUp),
            "slide-down" | "slidedown" => Some(TransitionKind::SlideDown),
            "zoomin" | "zoom-in" => Some(TransitionKind::ZoomIn),
            "smooth-left" | "smoothleft" => Some(TransitionKind::SmoothLeft),
            "smooth-right" | "smoothright" => Some(TransitionKind::SmoothRight),
            "smooth-up" | "smoothup" => Some(TransitionKind::SmoothUp),
            "smooth-down" | "smoothdown" => Some(TransitionKind::SmoothDown),
            "squeeze-h" | "squeezeh" => Some(TransitionKind::SqueezeH),
            "squeeze-v" | "squeezev" => Some(TransitionKind::SqueezeV),
            "circlecrop" | "circle-crop" => Some(TransitionKind::CircleCrop),
            "pixelize" => Some(TransitionKind::Pixelize),
            _ => None,
        }
    }
}

/// A fully resolved image overlay.
#[derive(Debug, Clone)]
pub struct IrImageOverlay {
    pub asset_name: String,
    pub asset_path: PathBuf,
    pub at_sec: f64,
    pub duration_sec: f64,
    pub position: Position,
    pub scale: Option<f64>,
    pub opacity: Option<f64>,
}
