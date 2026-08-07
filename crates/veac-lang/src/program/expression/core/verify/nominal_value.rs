use std::ops::Range;

use super::definitions::Definitions;
use super::{error, type_table};
use crate::program::expression::core::{CoreProgram, ValueId};
use crate::program::expression::{ExpressionError, ValueType, ValueTypeKind};
use crate::program::{
    FieldDefinition, FieldIndex, TypeDefinitionKind, TypeId, TypeRegistry, VariantIndex,
};

pub(super) fn structure(
    type_id: TypeId,
    fields: &[ValueId],
    program: &CoreProgram,
    definitions: &Definitions,
    registry: &TypeRegistry,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    let definition = definition(type_id, registry, span.clone())?;
    let TypeDefinitionKind::Struct(layout) = definition.kind() else {
        return Err(error("StructConstruct requires a struct TypeId", span));
    };
    verify_fields(fields, layout.fields(), program, definitions, span)?;
    Ok(ValueType::nominal(definition.type_ref().clone()))
}

pub(super) fn enumeration(
    type_id: TypeId,
    variant: VariantIndex,
    fields: &[ValueId],
    program: &CoreProgram,
    definitions: &Definitions,
    registry: &TypeRegistry,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    let definition = definition(type_id, registry, span.clone())?;
    let TypeDefinitionKind::Enum(layout) = definition.kind() else {
        return Err(error("EnumConstruct requires an enum TypeId", span));
    };
    let layout = layout
        .variants()
        .get(variant.index())
        .ok_or_else(|| error("EnumConstruct has an unknown variant index", span.clone()))?;
    verify_fields(fields, layout.fields(), program, definitions, span)?;
    Ok(ValueType::nominal(definition.type_ref().clone()))
}

pub(super) fn project(
    structure: ValueId,
    field: FieldIndex,
    program: &CoreProgram,
    definitions: &Definitions,
    registry: &TypeRegistry,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    let value_type = value_type(structure, program, definitions, span.clone())?;
    let ValueTypeKind::Nominal(reference) = value_type.kind() else {
        return Err(error("StructProject receiver must be nominal", span));
    };
    let definition = definition(reference.id(), registry, span.clone())?;
    let TypeDefinitionKind::Struct(layout) = definition.kind() else {
        return Err(error("StructProject receiver must be a struct", span));
    };
    layout
        .fields()
        .get(field.index())
        .map(|field| field.value_type().clone())
        .ok_or_else(|| error("StructProject has an unknown field index", span))
}

fn verify_fields(
    values: &[ValueId],
    layout: &[FieldDefinition],
    program: &CoreProgram,
    definitions: &Definitions,
    span: Range<usize>,
) -> Result<(), ExpressionError> {
    if values.len() != layout.len() {
        return Err(error(
            "nominal constructor field count does not match its layout",
            span,
        ));
    }
    for (value, field) in values.iter().zip(layout) {
        let actual = value_type(*value, program, definitions, span.clone())?;
        if actual != field.value_type() {
            return Err(error(
                format!(
                    "nominal field `{}` expects {}, found {actual}",
                    field.name(),
                    field.value_type()
                ),
                span,
            ));
        }
    }
    Ok(())
}

fn value_type<'a>(
    value: ValueId,
    program: &'a CoreProgram,
    definitions: &Definitions,
    span: Range<usize>,
) -> Result<&'a ValueType, ExpressionError> {
    type_table::value(program, definitions.type_id(value), span)
}

fn definition(
    id: TypeId,
    registry: &TypeRegistry,
    span: Range<usize>,
) -> Result<&crate::program::TypeDefinition, ExpressionError> {
    registry
        .definition(id)
        .ok_or_else(|| error(format!("unknown nominal TypeId {id}"), span))
}
