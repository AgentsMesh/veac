use crate::program::diagnostic::Diagnostic;
use crate::program::expression::ValueType;
use crate::program::model::{Scope, SurfaceFile};
use crate::program::{TypeRegistry, TypeSyntax};

pub(super) fn resolve(
    file: &SurfaceFile,
    scope: &Scope,
    syntax: &TypeSyntax,
) -> Result<ValueType, Diagnostic> {
    resolve_in(&file.path, &scope.types, syntax)
}

pub(super) fn resolve_in(
    path: &str,
    registry: &TypeRegistry,
    syntax: &TypeSyntax,
) -> Result<ValueType, Diagnostic> {
    syntax
        .resolve(&|name| registry.resolve(name).cloned())
        .map_err(|error| Diagnostic::new(error.code(), path, error.message(), error.span()))
}
