use super::{ColorStageDecl, Identifier, Span, Spanned};

#[derive(Debug, Clone, PartialEq)]
pub struct ColorModifierDecl {
    pub id: Identifier,
    pub input: Spanned<veac_ir::ColorSpace>,
    pub working: Spanned<veac_ir::ColorSpace>,
    pub output: Spanned<veac_ir::ColorSpace>,
    pub stages: Vec<ColorStageDecl>,
    pub span: Span,
}

impl Eq for ColorModifierDecl {}
