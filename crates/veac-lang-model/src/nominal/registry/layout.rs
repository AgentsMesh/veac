use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::{ValueType, ValueTypeKind};

use super::super::{TypeDefinition, TypeDefinitionKind, TypeId, TypeRegistryError};

type Definitions = BTreeMap<TypeId, Arc<TypeDefinition>>;

pub(super) fn verify(definitions: &Definitions) -> Result<(), TypeRegistryError> {
    let mut done = BTreeSet::new();
    for id in definitions.keys().copied() {
        visit(id, definitions, &mut Vec::new(), &mut done)?;
    }
    Ok(())
}

pub(super) fn contains_function(definitions: &Definitions, id: TypeId) -> Option<bool> {
    contains_definition(id, definitions, &mut BTreeMap::new())
}

fn visit(
    id: TypeId,
    definitions: &Definitions,
    active: &mut Vec<TypeId>,
    done: &mut BTreeSet<TypeId>,
) -> Result<(), TypeRegistryError> {
    if done.contains(&id) {
        return Ok(());
    }
    if let Some(start) = active.iter().position(|value| *value == id) {
        let names = canonical_cycle_names(&active[start..], definitions);
        return Err(error(
            "TYPE_RECURSIVE_LAYOUT",
            format!("recursive nominal layout: {}", names.join(" -> ")),
        ));
    }
    let definition = definitions.get(&id).ok_or_else(|| unknown(id))?;
    active.push(id);
    for value in field_types(definition) {
        visit_type(value, definitions, active, done)?;
    }
    active.pop();
    done.insert(id);
    Ok(())
}

fn canonical_cycle_names(cycle: &[TypeId], definitions: &Definitions) -> Vec<String> {
    let start = cycle
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            definitions[left]
                .declared_name()
                .cmp(definitions[right].declared_name())
                .then_with(|| left.cmp(right))
        })
        .map_or(0, |(index, _)| index);
    let mut names = cycle[start..]
        .iter()
        .chain(&cycle[..start])
        .map(|id| definitions[id].declared_name().to_owned())
        .collect::<Vec<_>>();
    names.push(names[0].clone());
    names
}

fn visit_type(
    value: &ValueType,
    definitions: &Definitions,
    active: &mut Vec<TypeId>,
    done: &mut BTreeSet<TypeId>,
) -> Result<(), TypeRegistryError> {
    match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => Ok(()),
        ValueTypeKind::Nominal(value) => visit(value.id(), definitions, active, done),
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => visit_type(value, definitions, active, done),
        ValueTypeKind::Tuple(values) => values
            .iter()
            .try_for_each(|value| visit_type(value, definitions, active, done)),
        ValueTypeKind::Function {
            parameters, result, ..
        } => parameters
            .iter()
            .chain(std::iter::once(result))
            .try_for_each(|value| visit_type(value, definitions, active, done)),
    }
}

fn contains_definition(
    id: TypeId,
    definitions: &Definitions,
    memo: &mut BTreeMap<TypeId, bool>,
) -> Option<bool> {
    if let Some(value) = memo.get(&id) {
        return Some(*value);
    }
    let definition = definitions.get(&id)?;
    let result = field_types(definition)
        .into_iter()
        .map(|value| contains_type(value, definitions, memo))
        .try_fold(false, |found, value| value.map(|value| found || value))?;
    memo.insert(id, result);
    Some(result)
}

fn contains_type(
    value: &ValueType,
    definitions: &Definitions,
    memo: &mut BTreeMap<TypeId, bool>,
) -> Option<bool> {
    match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => Some(false),
        ValueTypeKind::Nominal(value) => contains_definition(value.id(), definitions, memo),
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => contains_type(value, definitions, memo),
        ValueTypeKind::Tuple(values) => contains_sequence(values, definitions, memo),
        ValueTypeKind::Function { .. } => Some(true),
    }
}

fn contains_sequence(
    values: &[ValueType],
    definitions: &Definitions,
    memo: &mut BTreeMap<TypeId, bool>,
) -> Option<bool> {
    values
        .iter()
        .map(|value| contains_type(value, definitions, memo))
        .try_fold(false, |found, value| value.map(|value| found || value))
}

fn field_types(value: &TypeDefinition) -> Vec<&ValueType> {
    match value.kind() {
        TypeDefinitionKind::Struct(value) => value
            .fields()
            .iter()
            .map(|field| field.value_type())
            .collect(),
        TypeDefinitionKind::Enum(value) => value
            .variants()
            .iter()
            .flat_map(|variant| variant.fields())
            .map(|field| field.value_type())
            .collect(),
    }
}

fn unknown(id: TypeId) -> TypeRegistryError {
    error(
        "TYPE_UNKNOWN_ID",
        format!("nominal layout refers to unknown TypeId {id}"),
    )
}

fn error(code: &'static str, message: impl Into<String>) -> TypeRegistryError {
    TypeRegistryError::new(code, message)
}
