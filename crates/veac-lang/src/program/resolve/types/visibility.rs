use std::collections::{BTreeMap, BTreeSet};

use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{ValueType, ValueTypeKind};
use crate::program::model::{Scope, SurfaceFile, TypeDeclKind, TypeFieldDecl};
use crate::program::{TypeId, TypeSyntax};

use super::super::type_annotations;

pub(super) fn validate(
    file: &SurfaceFile,
    scope: &Scope,
    exported: &BTreeSet<String>,
) -> Result<(), Diagnostic> {
    let private = file
        .types
        .iter()
        .filter(|value| !exported.contains(&value.name))
        .map(|value| (TypeId::derive(&file.path, &value.name), value.name.as_str()))
        .collect::<BTreeMap<_, _>>();
    for declaration in file.types.iter().filter(|value| value.exported) {
        match &declaration.kind {
            TypeDeclKind::Struct(value) => fields(file, scope, &private, &value.fields)?,
            TypeDeclKind::Enum(value) => {
                for variant in &value.variants {
                    fields(file, scope, &private, &variant.fields)?;
                }
            }
        }
    }
    for declaration in file.functions.iter().filter(|value| value.exported) {
        for parameter in &declaration.parameters {
            syntax(file, scope, &private, &parameter.type_syntax)?;
        }
        syntax(file, scope, &private, &declaration.return_type_syntax)?;
    }
    for implementation in &file.implementations {
        for method in implementation.methods.iter().filter(|value| value.exported) {
            syntax(file, scope, &private, &implementation.target)?;
            for parameter in &method.parameters {
                syntax(file, scope, &private, &parameter.type_syntax)?;
            }
            syntax(file, scope, &private, &method.return_type_syntax)?;
        }
    }
    for declaration in file.constants.iter().filter(|value| value.exported) {
        syntax(file, scope, &private, &declaration.type_syntax)?;
    }
    Ok(())
}

fn fields(
    file: &SurfaceFile,
    scope: &Scope,
    private: &BTreeMap<TypeId, &str>,
    values: &[TypeFieldDecl],
) -> Result<(), Diagnostic> {
    for field in values {
        syntax(file, scope, private, &field.type_syntax)?;
    }
    Ok(())
}

fn syntax(
    file: &SurfaceFile,
    scope: &Scope,
    private: &BTreeMap<TypeId, &str>,
    syntax: &TypeSyntax,
) -> Result<(), Diagnostic> {
    let value = type_annotations::resolve(file, scope, syntax)?;
    value_type(file, private, &value, syntax.span())
}

fn value_type(
    file: &SurfaceFile,
    private: &BTreeMap<TypeId, &str>,
    value: &ValueType,
    span: crate::authoring::Span,
) -> Result<(), Diagnostic> {
    match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => Ok(()),
        ValueTypeKind::Nominal(value) => match private.get(&value.id()) {
            Some(name) => Err(Diagnostic::new(
                "PROGRAM_PRIVATE_TYPE_LEAK",
                &file.path,
                format!("public declaration exposes private type `{name}`"),
                span,
            )),
            None => Ok(()),
        },
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => value_type(file, private, value, span),
        ValueTypeKind::Tuple(values) => values
            .iter()
            .try_for_each(|value| value_type(file, private, value, span)),
        ValueTypeKind::Function {
            parameters, result, ..
        } => parameters
            .iter()
            .chain(std::iter::once(result))
            .try_for_each(|value| value_type(file, private, value, span)),
    }
}
