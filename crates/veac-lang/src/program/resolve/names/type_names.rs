use std::sync::Arc;

use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::model::Scope;
use crate::program::TypeRegistryBuilder;

use super::super::retained;

pub(super) fn namespace(
    path: &str,
    alias: &str,
    imported: &Scope,
    target: &mut Scope,
    span: Span,
    retained: &mut retained::Budget,
) -> Result<(), Diagnostic> {
    let mut builder = TypeRegistryBuilder::new();
    registry_result(builder.merge(&target.types), path, span)?;
    registry_result(builder.merge_definitions(&imported.types), path, span)?;
    for (name, value) in imported.types.names() {
        let qualified = format!("{alias}.{name}");
        if target.types.resolve(&qualified).is_some() {
            return Err(Diagnostic::new(
                "PROGRAM_DUPLICATE_TYPE",
                path,
                format!("imported type `{qualified}` is bound more than once"),
                span,
            ));
        }
        retained.alias(path, &qualified, span)?;
        registry_result(
            builder.bind(&qualified, value.with_diagnostic_name(qualified.clone())),
            path,
            span,
        )?;
    }
    target.types = Arc::new(registry_result(builder.finish(), path, span)?);
    Ok(())
}

pub(super) fn prelude(
    path: &str,
    imported: &Scope,
    target: &mut Scope,
    span: Span,
    retained: &mut retained::Budget,
) -> Result<(), Diagnostic> {
    let mut builder = TypeRegistryBuilder::new();
    registry_result(builder.merge(&target.types), path, span)?;
    registry_result(builder.merge_definitions(&imported.types), path, span)?;
    for (name, value) in imported.types.names() {
        if target.types.resolve(name).is_some() {
            return Err(Diagnostic::new(
                "PROGRAM_DUPLICATE_TYPE",
                path,
                format!("prelude type `{name}` is bound more than once"),
                span,
            ));
        }
        retained.alias(path, name, span)?;
        registry_result(
            builder.bind(name, value.with_diagnostic_name(name)),
            path,
            span,
        )?;
    }
    target.types = Arc::new(registry_result(builder.finish(), path, span)?);
    Ok(())
}

fn diagnostic(
    path: &str,
    error: crate::program::type_system::TypeRegistryError,
    span: Span,
) -> Diagnostic {
    Diagnostic::new(error.code(), path, error.message(), span)
}

fn registry_result<T>(
    result: Result<T, crate::program::type_system::TypeRegistryError>,
    path: &str,
    span: Span,
) -> Result<T, Diagnostic> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(diagnostic(path, error, span)),
    }
}
