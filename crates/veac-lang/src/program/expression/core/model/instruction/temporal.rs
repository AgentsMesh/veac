#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreTemporalComposeOperation {
    Vec2,
    Point,
    Rect,
    Color,
}

impl CoreTemporalComposeOperation {
    pub const fn arity(self) -> usize {
        match self {
            Self::Vec2 | Self::Point => 2,
            Self::Rect | Self::Color => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreTemporalProjectOperation {
    Vec2X,
    Vec2Y,
    PointX,
    PointY,
    RectX,
    RectY,
    RectWidth,
    RectHeight,
    ColorRed,
    ColorGreen,
    ColorBlue,
    ColorAlpha,
}
