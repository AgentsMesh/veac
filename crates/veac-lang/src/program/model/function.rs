use crate::authoring::Span;

use crate::program::syntax_document::SyntaxSlice;
use crate::program::TypeSyntax;

#[derive(Debug, Clone)]
pub(crate) struct FunctionDecl {
    pub name: String,
    pub parameters: Vec<FunctionParameterDecl>,
    pub return_type_syntax: TypeSyntax,
    pub body: FunctionBodyBinding,
    pub exported: bool,
    pub span: Span,
    pub syntax: SyntaxSlice,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionBodyBinding {
    pub span: Span,
    pub syntax: SyntaxSlice,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionParameterDecl {
    pub name: String,
    pub type_syntax: TypeSyntax,
    pub span: Span,
    pub default: Option<ParameterDefaultBinding>,
}

#[derive(Debug, Clone)]
pub(crate) struct ParameterDefaultBinding {
    pub span: Span,
    pub syntax: SyntaxSlice,
}
