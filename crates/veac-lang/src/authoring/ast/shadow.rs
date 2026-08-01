use super::{NumberLiteral, Spanned, VectorDecl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowDecl {
    pub color: Spanned<String>,
    pub opacity: NumberLiteral,
    pub blur: NumberLiteral,
    pub offset: VectorDecl,
}
