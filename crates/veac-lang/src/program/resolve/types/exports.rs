use std::collections::BTreeSet;
use std::sync::Arc;

use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{ValueType, ValueTypeKind};
use crate::program::model::{Scope, SurfaceFile};
use crate::program::{TypeDefinition, TypeDefinitionKind, TypeId, TypeSyntax};

use super::super::type_annotations;

pub(super) fn definitions(
    file: &SurfaceFile,
    scope: &Scope,
    exported_types: &BTreeSet<String>,
) -> Result<Vec<Arc<TypeDefinition>>, Diagnostic> {
    let mut roots = Vec::new();
    for name in exported_types {
        let reference = scope
            .types
            .resolve(name)
            .expect("exported type was resolved");
        roots.push(ValueType::nominal(reference.clone()));
    }
    collect_public_syntax_roots(file, scope, &mut roots)?;
    let mut retained = BTreeSet::new();
    for root in &roots {
        collect_type(root, scope, &mut retained, &file.path)?;
    }
    Ok(retained
        .into_iter()
        .map(|id| {
            scope
                .types
                .definition_handle(id)
                .expect("collected type belongs to resolved registry")
        })
        .collect())
}

fn collect_public_syntax_roots(
    file: &SurfaceFile,
    scope: &Scope,
    roots: &mut Vec<ValueType>,
) -> Result<(), Diagnostic> {
    for value in file.functions.iter().filter(|value| value.exported) {
        for parameter in &value.parameters {
            push(file, scope, &parameter.type_syntax, roots)?;
        }
        push(file, scope, &value.return_type_syntax, roots)?;
    }
    for implementation in &file.implementations {
        for method in implementation.methods.iter().filter(|value| value.exported) {
            push(file, scope, &implementation.target, roots)?;
            for parameter in &method.parameters {
                push(file, scope, &parameter.type_syntax, roots)?;
            }
            push(file, scope, &method.return_type_syntax, roots)?;
        }
    }
    for value in file.constants.iter().filter(|value| value.exported) {
        push(file, scope, &value.type_syntax, roots)?;
    }
    Ok(())
}

fn push(
    file: &SurfaceFile,
    scope: &Scope,
    syntax: &TypeSyntax,
    roots: &mut Vec<ValueType>,
) -> Result<(), Diagnostic> {
    roots.push(type_annotations::resolve(file, scope, syntax)?);
    Ok(())
}

fn collect_type(
    value: &ValueType,
    scope: &Scope,
    retained: &mut BTreeSet<TypeId>,
    path: &str,
) -> Result<(), Diagnostic> {
    match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => Ok(()),
        ValueTypeKind::Nominal(reference) => {
            if !retained.insert(reference.id()) {
                return Ok(());
            }
            let definition = scope.types.definition(reference.id()).ok_or_else(|| {
                Diagnostic::new(
                    "PROGRAM_UNKNOWN_TYPE",
                    path,
                    format!("public ABI refers to unknown type `{reference}`"),
                    Default::default(),
                )
            })?;
            match definition.kind() {
                TypeDefinitionKind::Struct(value) => {
                    collect_fields(value.fields(), scope, retained, path)
                }
                TypeDefinitionKind::Enum(value) => value
                    .variants()
                    .iter()
                    .try_for_each(|value| collect_fields(value.fields(), scope, retained, path)),
            }
        }
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => collect_type(value, scope, retained, path),
        ValueTypeKind::Tuple(values) => values
            .iter()
            .try_for_each(|value| collect_type(value, scope, retained, path)),
        ValueTypeKind::Function {
            parameters, result, ..
        } => parameters
            .iter()
            .chain(std::iter::once(result))
            .try_for_each(|value| collect_type(value, scope, retained, path)),
    }
}

fn collect_fields(
    fields: &[crate::program::FieldDefinition],
    scope: &Scope,
    retained: &mut BTreeSet<TypeId>,
    path: &str,
) -> Result<(), Diagnostic> {
    fields
        .iter()
        .try_for_each(|field| collect_type(field.value_type(), scope, retained, path))
}
