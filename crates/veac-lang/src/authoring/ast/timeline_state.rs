use super::{Span, Spanned};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementModeDecl {
    Free,
    Magnetic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStateDecl {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioStateDecl {
    Audible,
    Muted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationStateDecl {
    Normal,
    Solo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditingStateDecl {
    Editable,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackStateDecl {
    pub playback: Option<Spanned<PlaybackStateDecl>>,
    pub audio: Option<Spanned<AudioStateDecl>>,
    pub isolation: Option<Spanned<IsolationStateDecl>>,
    pub editing: Option<Spanned<EditingStateDecl>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStateDecl {
    pub playback: Spanned<PlaybackStateDecl>,
    pub span: Span,
}
