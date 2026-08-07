use crate::authoring::Span;

use crate::program::TypeSyntax;

#[derive(Debug, Clone)]
pub(crate) struct FunctionDecl {
    pub name: String,
    pub parameters: Vec<FunctionParameterDecl>,
    pub return_type_syntax: TypeSyntax,
    pub body: FunctionBodyBinding,
    pub exported: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionBodyBinding {
    pub source: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionParameterDecl {
    pub name: String,
    pub type_syntax: TypeSyntax,
    pub span: Span,
}
