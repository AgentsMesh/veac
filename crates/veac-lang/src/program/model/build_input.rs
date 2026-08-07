use crate::authoring::Span;
use crate::program::TypeSyntax;

#[derive(Debug, Clone)]
pub(crate) struct BuildInputDecl {
    pub role: crate::program::BuildInputRole,
    pub name: String,
    pub name_span: Span,
    pub type_syntax: TypeSyntax,
    pub span: Span,
}
