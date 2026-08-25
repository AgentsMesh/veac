use std::mem::{size_of, size_of_val};

use crate::program::model::{
    BuildInputDecl, ConstDecl, EnumDecl, FunctionDecl, ImplDecl, ImportDecl, SurfaceFile,
    TemporalDecl, TypeDecl, TypeDeclKind, TypeFieldDecl,
};
use crate::program::syntax_document::{SyntaxDocument, SyntaxElement, SyntaxSlice};
use crate::program::token::{Token, TokenKind};
use crate::program::TypeSyntax;

pub(super) fn bytes(file: &SurfaceFile) -> usize {
    let mut total = size_of_val(file)
        .saturating_add(file.path.len())
        .saturating_add(document(&file.syntax));
    total = total.saturating_add(file.imports.iter().map(import).sum());
    total = total.saturating_add(file.inputs.iter().map(input).sum());
    total = total.saturating_add(file.constants.iter().map(constant).sum());
    total = total.saturating_add(file.functions.iter().map(function).sum());
    total = total.saturating_add(file.implementations.iter().map(implementation).sum());
    total = total.saturating_add(file.types.iter().map(type_decl).sum());
    total.saturating_add(file.temporal.iter().map(temporal).sum())
}

fn document(value: &SyntaxDocument) -> usize {
    let tokens = value.tokens().iter().map(token).sum::<usize>();
    value
        .source()
        .len()
        .saturating_add(value.tokens().len().saturating_mul(size_of::<Token>()))
        .saturating_add(tokens)
        .saturating_add(
            value
                .elements()
                .len()
                .saturating_mul(size_of::<SyntaxElement>()),
        )
}

fn token(value: &Token) -> usize {
    match &value.kind {
        TokenKind::Word(value)
        | TokenKind::Number(value)
        | TokenKind::String(value)
        | TokenKind::Color(value)
        | TokenKind::LocalId(value) => value.len(),
        _ => 0,
    }
}

fn import(value: &ImportDecl) -> usize {
    size_of_val(value)
        .saturating_add(value.path.len())
        .saturating_add(value.alias.len())
}

fn input(value: &BuildInputDecl) -> usize {
    size_of_val(value)
        .saturating_add(value.name.len())
        .saturating_add(type_syntax(&value.type_syntax))
}

fn constant(value: &ConstDecl) -> usize {
    size_of_val(value)
        .saturating_add(value.name.len())
        .saturating_add(type_syntax(&value.type_syntax))
        .saturating_add(slice(&value.expression))
}

fn function(value: &FunctionDecl) -> usize {
    size_of_val(value)
        .saturating_add(value.name.len())
        .saturating_add(value.parameters.iter().map(parameter).sum::<usize>())
        .saturating_add(type_syntax(&value.return_type_syntax))
        .saturating_add(slice(&value.syntax))
        .saturating_add(slice(&value.body.syntax))
}

fn implementation(value: &ImplDecl) -> usize {
    size_of_val(value)
        .saturating_add(value.identity.len())
        .saturating_add(type_syntax(&value.target))
        .saturating_add(slice(&value.syntax))
        .saturating_add(value.methods.iter().map(method).sum::<usize>())
}

fn method(value: &crate::program::model::MethodDecl) -> usize {
    size_of_val(value)
        .saturating_add(value.name.len())
        .saturating_add(value.parameters.iter().map(parameter).sum::<usize>())
        .saturating_add(type_syntax(&value.return_type_syntax))
        .saturating_add(slice(&value.syntax))
        .saturating_add(slice(&value.body.syntax))
}

fn parameter(value: &crate::program::model::FunctionParameterDecl) -> usize {
    size_of_val(value)
        .saturating_add(value.name.len())
        .saturating_add(type_syntax(&value.type_syntax))
        .saturating_add(
            value
                .default
                .as_ref()
                .map_or(0, |default| slice(&default.syntax)),
        )
}

fn type_decl(value: &TypeDecl) -> usize {
    size_of_val(value)
        .saturating_add(value.name.len())
        .saturating_add(match &value.kind {
            TypeDeclKind::Struct(value) => struct_decl(value),
            TypeDeclKind::Enum(value) => enum_decl(value),
        })
}

fn struct_decl(value: &crate::program::model::StructDecl) -> usize {
    size_of_val(value).saturating_add(value.fields.iter().map(field).sum::<usize>())
}

fn enum_decl(value: &EnumDecl) -> usize {
    size_of_val(value).saturating_add(
        value
            .variants
            .iter()
            .map(|variant| {
                size_of_val(variant)
                    .saturating_add(variant.name.len())
                    .saturating_add(variant.fields.iter().map(field).sum::<usize>())
            })
            .sum::<usize>(),
    )
}

fn field(value: &TypeFieldDecl) -> usize {
    size_of_val(value)
        .saturating_add(value.name.len())
        .saturating_add(type_syntax(&value.type_syntax))
}

fn temporal(value: &TemporalDecl) -> usize {
    size_of_val(value)
        .saturating_add(value.source.as_ref().map_or(0, |path| {
            path.project.len().saturating_add(path.resource.len())
        }))
        .saturating_add(slice(&value.syntax))
        .saturating_add(slice(&value.body.syntax))
        .saturating_add(target_bytes(&value.target))
}

fn target_bytes(value: &crate::program::model::TemporalTarget) -> usize {
    let paths = value
        .item()
        .map(|path| path.segments().iter().map(|part| part.len()).sum())
        .unwrap_or(0)
        .saturating_add(value.apply().map_or(0, |path| {
            path.segments().iter().map(|part| part.len()).sum()
        }));
    paths.saturating_add(match value {
        crate::program::model::TemporalTarget::ClipEffect {
            effect, parameter, ..
        } => effect.len().saturating_add(parameter.len()),
        crate::program::model::TemporalTarget::ApplyEffect {
            stage,
            effect,
            parameter,
            ..
        } => stage
            .len()
            .saturating_add(effect.len())
            .saturating_add(parameter.len()),
        _ => 0,
    })
}

fn slice(value: &SyntaxSlice) -> usize {
    size_of_val(value)
}

fn type_syntax(value: &TypeSyntax) -> usize {
    size_of_val(value).saturating_add(match value.kind() {
        crate::program::TypeSyntaxKind::Primitive(_) => 0,
        crate::program::TypeSyntaxKind::Named(name) => name.len(),
        crate::program::TypeSyntaxKind::List(value)
        | crate::program::TypeSyntaxKind::Range(value) => type_syntax(value),
        crate::program::TypeSyntaxKind::Map { key, value } => {
            type_syntax(key).saturating_add(type_syntax(value))
        }
        crate::program::TypeSyntaxKind::Tuple(values) => values.iter().map(type_syntax).sum(),
        crate::program::TypeSyntaxKind::Function {
            parameters, result, ..
        } => parameters.iter().map(type_syntax).sum::<usize>() + type_syntax(result),
    })
}
