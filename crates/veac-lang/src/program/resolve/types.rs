use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::program::diagnostic::Diagnostic;
use crate::program::model::{Scope, SurfaceFile};
use crate::program::{TypeId, TypeRef, TypeRegistryBuilder};

use super::retained;

mod definition;
mod exports;
mod visibility;

pub(super) fn resolve(
    file: &SurfaceFile,
    scope: &mut Scope,
    retained: &mut retained::Budget,
) -> Result<BTreeSet<String>, Diagnostic> {
    let mut local = BTreeMap::new();
    for declaration in &file.types {
        if scope.types.resolve(&declaration.name).is_some()
            || local
                .insert(
                    declaration.name.clone(),
                    TypeRef::new(
                        TypeId::derive(&file.path, &declaration.name),
                        declaration.name.clone(),
                    ),
                )
                .is_some()
        {
            return Err(duplicate(file, declaration));
        }
    }
    let mut builder = TypeRegistryBuilder::new();
    registry_result(builder.merge(&scope.types), file, first_span(file))?;
    for (name, reference) in &local {
        registry_result(
            builder.bind(name, reference.clone()),
            file,
            declaration_span(file, name),
        )?;
    }
    for declaration in &file.types {
        let value = Arc::new(definition::resolve(file, declaration, &|name| {
            local
                .get(name)
                .cloned()
                .or_else(|| scope.types.resolve(name).cloned())
        })?);
        retained.type_definition(file, value.as_ref())?;
        registry_result(builder.insert(value), file, declaration.span)?;
    }
    scope.types = Arc::new(registry_result(builder.finish(), file, first_span(file))?);
    Ok(file
        .types
        .iter()
        .filter(|value| value.exported)
        .map(|value| value.name.clone())
        .collect())
}

pub(super) fn validate_exports(
    file: &SurfaceFile,
    scope: &Scope,
    exported: &BTreeSet<String>,
) -> Result<(), Diagnostic> {
    visibility::validate(file, scope, exported)
}

pub(super) fn retain_exports(
    file: &SurfaceFile,
    scope: &mut Scope,
    exported: &BTreeSet<String>,
) -> Result<(), Diagnostic> {
    let definitions = exports::definitions(file, scope, exported)?;
    let mut builder = TypeRegistryBuilder::new();
    for definition in definitions {
        registry_result(builder.insert(definition), file, first_span(file))?;
    }
    for name in exported {
        let reference = scope
            .types
            .resolve(name)
            .expect("exported type was resolved")
            .clone();
        registry_result(
            builder.bind(name, reference),
            file,
            declaration_span(file, name),
        )?;
    }
    scope.types = Arc::new(registry_result(builder.finish(), file, first_span(file))?);
    Ok(())
}

fn duplicate(file: &SurfaceFile, declaration: &crate::program::model::TypeDecl) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_DUPLICATE_TYPE",
        &file.path,
        format!("type `{}` is declared more than once", declaration.name),
        declaration.name_span,
    )
}

fn registry_error(
    file: &SurfaceFile,
    error: crate::program::type_system::TypeRegistryError,
    span: crate::authoring::Span,
) -> Diagnostic {
    Diagnostic::new(error.code(), &file.path, error.message(), span)
}

fn registry_result<T>(
    result: Result<T, crate::program::type_system::TypeRegistryError>,
    file: &SurfaceFile,
    span: crate::authoring::Span,
) -> Result<T, Diagnostic> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(registry_error(file, error, span)),
    }
}

fn declaration_span(file: &SurfaceFile, name: &str) -> crate::authoring::Span {
    file.types
        .iter()
        .find(|value| value.name == name)
        .map(|value| value.name_span)
        .unwrap_or_default()
}

fn first_span(file: &SurfaceFile) -> crate::authoring::Span {
    file.types
        .first()
        .map(|value| value.span)
        .unwrap_or_default()
}
