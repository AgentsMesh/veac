use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;

use crate::program::expression::core::{
    CoreClosureDefinition, CoreNominalDefinition, CoreType, CoreTypeTable,
};
use crate::program::expression::{ValueType, ValueTypeKind};
use crate::program::{TypeDefinition, TypeDefinitionKind, TypeId, TypeRegistry};

pub(super) fn collect(
    types: &CoreTypeTable,
    closures: &[CoreClosureDefinition],
    extra_types: &[ValueType],
    registry: &TypeRegistry,
    span: Range<usize>,
) -> Vec<CoreNominalDefinition> {
    let mut output = BTreeMap::new();
    for entry in types.entries() {
        if let CoreType::Value(value) = entry.kind() {
            collect_type(value, registry, &mut output);
        }
    }
    for closure in closures {
        closure
            .parameter_types()
            .iter()
            .chain(closure.capture_types())
            .for_each(|value| collect_type(value, registry, &mut output));
    }
    for value in extra_types {
        collect_type(value, registry, &mut output);
    }
    output
        .into_values()
        .map(|definition| CoreNominalDefinition::new(definition, span.clone()))
        .collect()
}

fn collect_type(
    value: &ValueType,
    registry: &TypeRegistry,
    output: &mut BTreeMap<TypeId, Arc<TypeDefinition>>,
) {
    match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => {}
        ValueTypeKind::Nominal(value) => collect_definition(value.id(), registry, output),
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => collect_type(value, registry, output),
        ValueTypeKind::Tuple(values) => values
            .iter()
            .for_each(|value| collect_type(value, registry, output)),
        ValueTypeKind::Function {
            parameters, result, ..
        } => parameters
            .iter()
            .chain(std::iter::once(result))
            .for_each(|value| collect_type(value, registry, output)),
    }
}

fn collect_definition(
    id: TypeId,
    registry: &TypeRegistry,
    output: &mut BTreeMap<TypeId, Arc<TypeDefinition>>,
) {
    if output.contains_key(&id) {
        return;
    }
    let definition = registry
        .definition_handle(id)
        .expect("typed nominal reference must belong to its HIR registry");
    output.insert(id, Arc::clone(&definition));
    match definition.kind() {
        TypeDefinitionKind::Struct(value) => value
            .fields()
            .iter()
            .for_each(|field| collect_type(field.value_type(), registry, output)),
        TypeDefinitionKind::Enum(value) => value.variants().iter().for_each(|variant| {
            variant
                .fields()
                .iter()
                .for_each(|field| collect_type(field.value_type(), registry, output));
        }),
    }
}
