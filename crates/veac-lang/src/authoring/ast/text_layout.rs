use super::{NumberLiteral, Span, Spanned, VectorDecl};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextLayoutDecl {
    pub box_width: Option<NumberLiteral>,
    pub box_height: Option<NumberLiteral>,
    pub wrap: Option<Spanned<veac_ir::TextWrap>>,
    pub overflow: Option<Spanned<veac_ir::TextOverflow>>,
    pub horizontal_alignment: Option<Spanned<veac_ir::HorizontalTextAlignment>>,
    pub vertical_alignment: Option<Spanned<veac_ir::VerticalTextAlignment>>,
    pub writing_mode: Option<Spanned<veac_ir::TextWritingMode>>,
    pub orientation: Option<Spanned<veac_ir::TextOrientation>>,
    pub path: Option<TextPathDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPathDecl {
    pub points: Vec<VectorDecl>,
    pub start_offset: NumberLiteral,
    pub reverse: Spanned<bool>,
    pub align: Spanned<veac_ir::TextPathAlignment>,
    pub span: Span,
}
