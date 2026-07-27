use super::{Identifier, NumberLiteral, ParameterDecl, Span, Spanned, VectorDecl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskModifierDecl {
    pub id: Identifier,
    pub shape: MaskShapeDecl,
    pub position: Option<ParameterDecl<VectorDecl>>,
    pub scale: Option<ParameterDecl<VectorDecl>>,
    pub rotation: Option<ParameterDecl<NumberLiteral>>,
    pub feather: Option<ParameterDecl<NumberLiteral>>,
    pub expansion: Option<ParameterDecl<NumberLiteral>>,
    pub invert: Option<Spanned<bool>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaskShapeDecl {
    Linear,
    Mirror,
    Circle,
    Rectangle,
    RoundedRectangle { radius: NumberLiteral, span: Span },
    Ellipse,
    Polygon { points: Vec<VectorDecl>, span: Span },
    Heart,
    Star,
    Path { points: Vec<VectorDecl>, span: Span },
}
