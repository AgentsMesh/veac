use std::collections::BTreeMap;
use std::sync::Arc;

use crate::authoring::Span;

use super::expression::{ExpressionContext, FunctionMap, Value};
use super::syntax_document::{SyntaxDocument, SyntaxSlice};
use super::{MethodRegistry, TypeRegistry, TypeSyntax};

mod function;
pub(crate) use function::{
    FunctionBodyBinding, FunctionDecl, FunctionParameterDecl, ParameterDefaultBinding,
};
mod method;
pub(crate) use method::{ImplDecl, MethodDecl};
mod type_declaration;
pub(crate) use type_declaration::{
    EnumDecl, EnumVariantDecl, StructDecl, TypeDecl, TypeDeclKind, TypeFieldDecl,
};
mod temporal;
pub(crate) use temporal::{
    TemporalApplyPath, TemporalDecl, TemporalItemPath, TemporalProperty, TemporalResourcePath,
    TemporalTarget,
};
mod build_input;
pub(crate) use build_input::BuildInputDecl;

#[derive(Debug, Clone)]
pub(crate) struct SurfaceFile {
    pub path: String,
    pub syntax: SyntaxDocument,
    pub kind: FileKind,
    pub imports: Vec<ImportDecl>,
    pub inputs: Vec<BuildInputDecl>,
    pub constants: Vec<ConstDecl>,
    pub functions: Vec<FunctionDecl>,
    pub implementations: Vec<ImplDecl>,
    pub types: Vec<TypeDecl>,
    pub temporal: Vec<TemporalDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FileKind {
    Entry,
    Module,
}

#[derive(Debug, Clone)]
pub(crate) struct ImportDecl {
    pub path: String,
    pub alias: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct ConstDecl {
    pub name: String,
    pub type_syntax: TypeSyntax,
    pub expression: SyntaxSlice,
    pub expression_span: Span,
    pub exported: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct RawBlock {
    pub span: Span,
    pub syntax: SyntaxSlice,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Scope {
    pub values: Arc<ValueMap>,
    pub functions: Arc<FunctionMap>,
    pub methods: Arc<MethodRegistry>,
    pub types: Arc<TypeRegistry>,
    pub build_inputs: Arc<BTreeMap<String, crate::program::BuildInputDeclaration>>,
}

pub(crate) type ValueMap = BTreeMap<String, Arc<Value>>;

impl SurfaceFile {
    pub(crate) fn source(&self) -> &str {
        self.syntax.source()
    }
}

impl Scope {
    pub(crate) fn expression_context(&self) -> ExpressionContext {
        ExpressionContext::empty()
            .with_types(Arc::clone(&self.types))
            .with_functions_arc(Arc::clone(&self.functions))
            .with_methods_arc(Arc::clone(&self.methods))
            .with_static_values(Arc::clone(&self.values))
            .with_build_inputs(
                self.build_inputs
                    .values()
                    .map(|input| (input.name().to_owned(), input.expression_slot()))
                    .collect(),
            )
    }
}
