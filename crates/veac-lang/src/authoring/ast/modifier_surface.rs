use super::{Identifier, NumberLiteral, ShadowDecl, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceModifierDecl {
    pub id: Identifier,
    pub corner_radius: NumberLiteral,
    pub shadow: Option<ShadowDecl>,
    pub span: Span,
}
