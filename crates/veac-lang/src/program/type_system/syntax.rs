use std::sync::Arc;

use crate::authoring::Span;
use veac_lang_model::{FunctionEffect, PrimitiveType, ValueType};

use super::TypeRef;

mod display;
mod resolve;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSyntax {
    kind: TypeSyntaxKind,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSyntaxKind {
    Primitive(PrimitiveType),
    Named(Arc<str>),
    List(Arc<TypeSyntax>),
    Range(Arc<TypeSyntax>),
    Map {
        key: Arc<TypeSyntax>,
        value: Arc<TypeSyntax>,
    },
    Tuple(Arc<[TypeSyntax]>),
    Function {
        parameters: Arc<[TypeSyntax]>,
        result: Arc<TypeSyntax>,
        effect: FunctionEffect,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSyntaxError {
    code: &'static str,
    message: String,
    span: Span,
}

impl TypeSyntax {
    pub const fn new(kind: TypeSyntaxKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub const fn kind(&self) -> &TypeSyntaxKind {
        &self.kind
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub fn resolve(
        &self,
        names: &dyn Fn(&str) -> Option<TypeRef>,
    ) -> Result<ValueType, TypeSyntaxError> {
        resolve::value(self, names)
    }
}

impl TypeSyntaxError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

impl std::fmt::Display for TypeSyntaxError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TypeSyntaxError {}
