use crate::authoring::Span;
use crate::program::syntax_document::SyntaxSlice;
use crate::program::TypeSyntax;

use super::{FunctionBodyBinding, FunctionParameterDecl};

#[derive(Debug, Clone)]
pub(crate) struct ImplDecl {
    pub target: TypeSyntax,
    pub identity: String,
    pub identity_span: Span,
    pub methods: Vec<MethodDecl>,
    pub span: Span,
    pub syntax: SyntaxSlice,
}

#[derive(Debug, Clone)]
pub(crate) struct MethodDecl {
    pub name: String,
    pub parameters: Vec<FunctionParameterDecl>,
    pub return_type_syntax: TypeSyntax,
    pub body: FunctionBodyBinding,
    pub exported: bool,
    pub span: Span,
    pub syntax: SyntaxSlice,
}
