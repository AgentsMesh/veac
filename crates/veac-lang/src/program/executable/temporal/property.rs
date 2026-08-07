use veac_ir::TemporalType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ClipTemporalProperty {
    VisualPosition,
    VisualScale,
    VisualRotation,
    VisualCrop,
    VisualOpacity,
    AudioGain,
    AudioPan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum MaskTemporalProperty {
    Position,
    Scale,
    Rotation,
    Feather,
    Expansion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TextTemporalProperty {
    Position,
    Scale,
    Rotation,
    Reveal,
    HighlightProgress,
    Opacity,
}

impl ClipTemporalProperty {
    pub(crate) const fn expected_type(self) -> TemporalType {
        match self {
            Self::VisualPosition => TemporalType::Point,
            Self::VisualScale => TemporalType::Vec2,
            Self::VisualRotation => TemporalType::Angle,
            Self::VisualCrop => TemporalType::Rect,
            Self::VisualOpacity | Self::AudioGain | Self::AudioPan => TemporalType::Scalar,
        }
    }

    pub(super) const fn opcode(self) -> u8 {
        match self {
            Self::VisualPosition => 0,
            Self::VisualScale => 1,
            Self::VisualRotation => 2,
            Self::VisualCrop => 3,
            Self::VisualOpacity => 4,
            Self::AudioGain => 5,
            Self::AudioPan => 6,
        }
    }
}

impl MaskTemporalProperty {
    pub(super) const fn expected_type(self) -> TemporalType {
        match self {
            Self::Position | Self::Scale => TemporalType::Vec2,
            Self::Rotation => TemporalType::Angle,
            Self::Feather | Self::Expansion => TemporalType::Scalar,
        }
    }

    pub(super) const fn opcode(self) -> u8 {
        match self {
            Self::Position => 0,
            Self::Scale => 1,
            Self::Rotation => 2,
            Self::Feather => 3,
            Self::Expansion => 4,
        }
    }
}

impl TextTemporalProperty {
    pub(super) const fn expected_type(self) -> TemporalType {
        match self {
            Self::Position => TemporalType::Point,
            Self::Scale => TemporalType::Vec2,
            Self::Rotation => TemporalType::Angle,
            Self::Reveal | Self::HighlightProgress | Self::Opacity => TemporalType::Scalar,
        }
    }

    pub(super) const fn opcode(self) -> u8 {
        match self {
            Self::Position => 0,
            Self::Scale => 1,
            Self::Rotation => 2,
            Self::Reveal => 3,
            Self::HighlightProgress => 4,
            Self::Opacity => 5,
        }
    }
}
