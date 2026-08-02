use super::{Identifier, NumberLiteral, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterDecl<T> {
    Constant(T),
    Curve {
        keys: Vec<ParameterKey<T>>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterKey<T> {
    pub id: Identifier,
    pub at: NumberLiteral,
    pub value: T,
    pub interpolation: InterpolationDecl,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpolationDecl {
    pub kind: InterpolationKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpolationKind {
    Hold,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier {
        x1: NumberLiteral,
        y1: NumberLiteral,
        x2: NumberLiteral,
        y2: NumberLiteral,
    },
    Spring {
        frequency: NumberLiteral,
        decay: NumberLiteral,
        initial_velocity: NumberLiteral,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointDecl {
    pub x: NumberLiteral,
    pub y: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorDecl {
    pub x: NumberLiteral,
    pub y: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RectDecl {
    pub x: NumberLiteral,
    pub y: NumberLiteral,
    pub width: NumberLiteral,
    pub height: NumberLiteral,
    pub span: Span,
}
