use std::collections::BTreeSet;

use super::error;
use crate::program::expression::core::{CoreProgram, CoreType};
use crate::program::expression::{ExpressionError, ValueType, ValueTypeKind};
use crate::program::{TypeDefinitionKind, TypeId, TypeRegistry, TypeRegistryBuilder};

#[cfg(test)]
mod tests;

pub(super) fn verify(
    program: &CoreProgram,
    parameters: &[ValueType],
    captures: &[ValueType],
) -> Result<TypeRegistry, ExpressionError> {
    let mut builder = TypeRegistryBuilder::new();
    let mut previous = None;
    for entry in &program.nominal_definitions {
        if entry.span.start > entry.span.end {
            return Err(error(
                "Core nominal definition has an invalid span",
                entry.span(),
            ));
        }
        let id = entry.definition().type_ref().id();
        if previous.is_some_and(|previous| previous >= id) {
            return Err(error(
                "Core nominal definitions must be unique and ordered by TypeId",
                entry.span(),
            ));
        }
        previous = Some(id);
        builder
            .insert(entry.definition_handle())
            .map_err(|failure| {
                error(
                    format!(
                        "Core nominal definition is invalid: {}: {}",
                        failure.code(),
                        failure.message()
                    ),
                    entry.span(),
                )
            })?;
    }
    let registry = builder.finish().map_err(|failure| {
        error(
            format!(
                "Core nominal registry is invalid: {}: {}",
                failure.code(),
                failure.message()
            ),
            0..0,
        )
    })?;
    require_exact_references(program, parameters, captures, &registry)?;
    Ok(registry)
}

fn require_exact_references(
    program: &CoreProgram,
    parameters: &[ValueType],
    captures: &[ValueType],
    registry: &TypeRegistry,
) -> Result<(), ExpressionError> {
    let mut referenced = BTreeSet::new();
    for entry in program.types.entries() {
        if let CoreType::Value(value) = entry.kind() {
            collect_type(value, registry, &mut referenced)?;
        }
    }
    for value in parameters.iter().chain(captures) {
        collect_type(value, registry, &mut referenced)?;
    }
    for closure in &program.closure_definitions {
        for value in closure
            .parameter_types()
            .iter()
            .chain(closure.capture_types())
        {
            collect_type(value, registry, &mut referenced)?;
        }
    }
    let embedded = registry
        .definitions()
        .map(|definition| definition.type_ref().id())
        .collect::<BTreeSet<_>>();
    if referenced == embedded {
        Ok(())
    } else {
        Err(error(
            "Core nominal definition table must equal its transitive type references",
            0..0,
        ))
    }
}

fn collect_type(
    value: &ValueType,
    registry: &TypeRegistry,
    output: &mut BTreeSet<TypeId>,
) -> Result<(), ExpressionError> {
    match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => Ok(()),
        ValueTypeKind::Nominal(value) => collect_definition(value.id(), registry, output),
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => collect_type(value, registry, output),
        ValueTypeKind::Tuple(values) => values
            .iter()
            .try_for_each(|value| collect_type(value, registry, output)),
        ValueTypeKind::Function {
            parameters, result, ..
        } => parameters
            .iter()
            .chain(std::iter::once(result))
            .try_for_each(|value| collect_type(value, registry, output)),
    }
}

fn collect_definition(
    id: TypeId,
    registry: &TypeRegistry,
    output: &mut BTreeSet<TypeId>,
) -> Result<(), ExpressionError> {
    if !output.insert(id) {
        return Ok(());
    }
    let definition = registry.definition(id).ok_or_else(|| {
        error(
            format!("Core value type refers to unknown TypeId {id}"),
            0..0,
        )
    })?;
    match definition.kind() {
        TypeDefinitionKind::Struct(value) => value
            .fields()
            .iter()
            .try_for_each(|field| collect_type(field.value_type(), registry, output)),
        TypeDefinitionKind::Enum(value) => value.variants().iter().try_for_each(|variant| {
            variant
                .fields()
                .iter()
                .try_for_each(|field| collect_type(field.value_type(), registry, output))
        }),
    }
}
