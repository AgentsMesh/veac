use std::collections::BTreeSet;
use std::sync::Arc;

use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{FunctionOrigin, FunctionParameter, ValueTypeKind};
use crate::program::model::{MethodDecl, Scope, SurfaceFile};
use crate::program::{
    MethodBody, MethodDefinition, MethodRegistryBuilder, MethodVisibility, TypeId,
};

use super::{retained, type_annotations};

pub(super) fn declare(
    file: &SurfaceFile,
    scope: &mut Scope,
    retained: &mut retained::Budget,
) -> Result<(), Diagnostic> {
    let mut builder = MethodRegistryBuilder::new();
    registry_result(
        builder.merge(&scope.methods, &scope.types),
        file,
        first_span(file),
    )?;
    let mut implementation_ids = BTreeSet::new();
    for implementation in &file.implementations {
        let receiver = type_annotations::resolve(file, scope, &implementation.target)?;
        let ValueTypeKind::Nominal(receiver) = receiver.kind() else {
            return Err(Diagnostic::new(
                "PROGRAM_IMPL_TARGET",
                &file.path,
                "implementation target must be a nominal type",
                implementation.target.span(),
            ));
        };
        require_local_receiver(file, receiver.id(), implementation.target.span())?;
        let identity = (receiver.id(), implementation.identity.clone());
        if !implementation_ids.insert(identity) {
            return Err(Diagnostic::new(
                "PROGRAM_DUPLICATE_IMPL_ID",
                &file.path,
                format!(
                    "implementation identity `@{}` is already used for `{}`",
                    implementation.identity, implementation.target
                ),
                implementation.identity_span,
            ));
        }
        for method in &implementation.methods {
            let definition = Arc::new(resolve_method(file, scope, receiver.clone(), method)?);
            retained.method_definition(file, definition.as_ref(), method.span)?;
            registry_result(builder.insert(definition, &scope.types), file, method.span)?;
        }
    }
    scope.methods = Arc::new(builder.finish());
    Ok(())
}

pub(super) fn retain_exports(scope: &mut Scope) {
    scope.methods = Arc::new(scope.methods.exported());
}

fn resolve_method(
    file: &SurfaceFile,
    scope: &Scope,
    receiver: crate::program::TypeRef,
    method: &MethodDecl,
) -> Result<MethodDefinition, Diagnostic> {
    let parameters = method
        .parameters
        .iter()
        .map(|parameter| {
            type_annotations::resolve(file, scope, &parameter.type_syntax)
                .map(|value| FunctionParameter::new(&parameter.name, value))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result = type_annotations::resolve(file, scope, &method.return_type_syntax)?;
    let signature =
        crate::program::MethodSignature::new(receiver, method.name.as_str(), parameters, result);
    let visibility = if method.exported {
        MethodVisibility::Exported
    } else {
        MethodVisibility::Private
    };
    Ok(
        MethodDefinition::new(signature, file.path.as_str(), visibility).with_body(
            MethodBody::new(
                method.body.source.as_str(),
                FunctionOrigin::new(&file.path, method.body.span.start..method.body.span.end),
            ),
        ),
    )
}

fn require_local_receiver(
    file: &SurfaceFile,
    receiver: TypeId,
    span: crate::authoring::Span,
) -> Result<(), Diagnostic> {
    if file
        .types
        .iter()
        .any(|value| TypeId::derive(&file.path, &value.name) == receiver)
    {
        Ok(())
    } else {
        Err(Diagnostic::new(
            "PROGRAM_FOREIGN_IMPL",
            &file.path,
            "implementation target must be declared in the same source",
            span,
        ))
    }
}

fn diagnostic(
    file: &SurfaceFile,
    error: crate::program::MethodRegistryError,
    span: crate::authoring::Span,
) -> Diagnostic {
    Diagnostic::new(error.code(), &file.path, error.message(), span)
}

fn registry_result<T>(
    result: Result<T, crate::program::MethodRegistryError>,
    file: &SurfaceFile,
    span: crate::authoring::Span,
) -> Result<T, Diagnostic> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(diagnostic(file, error, span)),
    }
}

fn first_span(file: &SurfaceFile) -> crate::authoring::Span {
    file.implementations
        .first()
        .map(|value| value.span)
        .unwrap_or_default()
}
