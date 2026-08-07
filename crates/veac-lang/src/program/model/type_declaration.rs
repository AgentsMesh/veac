use crate::authoring::Span;
use crate::program::TypeSyntax;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeDecl {
    pub name: String,
    pub name_span: Span,
    pub kind: TypeDeclKind,
    pub exported: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypeDeclKind {
    Struct(StructDecl),
    Enum(EnumDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StructDecl {
    pub fields: Vec<TypeFieldDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumDecl {
    pub variants: Vec<EnumVariantDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumVariantDecl {
    pub name: String,
    pub name_span: Span,
    pub fields: Vec<TypeFieldDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeFieldDecl {
    pub name: String,
    pub name_span: Span,
    pub type_syntax: TypeSyntax,
    pub span: Span,
}
