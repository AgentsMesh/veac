use super::{NumberLiteral, ParameterDecl, PointDecl, Span, Spanned, VectorDecl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextAnimationDecl {
    pub unit: Spanned<TextGranularityDecl>,
    pub stagger: NumberLiteral,
    pub reveal: Option<ParameterDecl<NumberLiteral>>,
    pub highlight: Option<TextHighlightDecl>,
    pub opacity: Option<ParameterDecl<NumberLiteral>>,
    pub transform: Option<TextAnimationTransformDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextGranularityDecl {
    Whole,
    Line,
    Word,
    Grapheme,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextHighlightDecl {
    pub fill: Spanned<String>,
    pub progress: ParameterDecl<NumberLiteral>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextAnimationTransformDecl {
    pub position: Option<ParameterDecl<PointDecl>>,
    pub scale: Option<ParameterDecl<VectorDecl>>,
    pub rotation: Option<ParameterDecl<NumberLiteral>>,
}
