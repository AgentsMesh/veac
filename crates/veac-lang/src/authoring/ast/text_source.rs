use super::{
    Identifier, NumberLiteral, ShadowDecl, Span, Spanned, TextAnimationDecl, TextLayoutDecl,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSourceDecl {
    pub content: Spanned<String>,
    pub style: TextStyleDecl,
    pub layout: TextLayoutDecl,
    pub animation: Option<TextAnimationDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptionSourceDecl {
    pub text: TextSourceDecl,
    pub speaker: Option<Spanned<String>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextStyleDecl {
    pub font: Option<TextFontDecl>,
    pub fallback_fonts: Vec<TextFontDecl>,
    pub size: Option<NumberLiteral>,
    pub weight: Option<Spanned<veac_ir::FontWeight>>,
    pub font_style: Option<Spanned<veac_ir::FontStyle>>,
    pub fill: Option<Spanned<String>>,
    pub tracking: Option<NumberLiteral>,
    pub line_height: Option<NumberLiteral>,
    pub background: Option<TextBackgroundDecl>,
    pub outline: Option<TextOutlineDecl>,
    pub shadow: Option<ShadowDecl>,
    pub spans: Vec<TextSpanDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextFontDecl {
    Family(Spanned<String>),
    Resource(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSpanDecl {
    pub start: NumberLiteral,
    pub end: NumberLiteral,
    pub font: Option<TextFontDecl>,
    pub size: Option<NumberLiteral>,
    pub weight: Option<Spanned<veac_ir::FontWeight>>,
    pub font_style: Option<Spanned<veac_ir::FontStyle>>,
    pub fill: Option<Spanned<String>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBackgroundDecl {
    pub color: Spanned<String>,
    pub padding: NumberLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextOutlineDecl {
    pub color: Spanned<String>,
    pub width: NumberLiteral,
}
