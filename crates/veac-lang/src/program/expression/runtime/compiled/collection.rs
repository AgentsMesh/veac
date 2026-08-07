use super::slot::{
    public, take_builder, take_pending, MapBuilderState, MapKeyIdentity, MapPendingState,
    RuntimeValue,
};
use super::value::contract;
use super::Evaluator;
use crate::program::expression::core::{CoreInstruction, CoreProgram, CoreType, CoreTypeId};
use crate::program::expression::{ExpressionError, MapValue, MapValueEntry, Value, ValueTypeKind};
use std::collections::BTreeSet;
use std::sync::Arc;

impl Evaluator<'_> {
    pub(super) fn sequence(
        &self,
        elements: &[crate::program::expression::ValueId],
        tuple: bool,
        instruction: &CoreInstruction,
        program: &CoreProgram,
        values: &[Option<RuntimeValue>],
    ) -> Result<RuntimeValue, ExpressionError> {
        self.execution
            .reserve_sequence_collection(elements.len(), instruction.span.clone())?;
        let values = elements
            .iter()
            .map(|id| public(values, *id, instruction))
            .collect::<Result<Vec<_>, _>>()?;
        let value = if tuple {
            Value::tuple(values)
        } else {
            let expected = program
                .value_type(instruction.type_id)
                .ok_or_else(|| contract("Core list result type is unavailable", instruction))?;
            let ValueTypeKind::List(element) = expected.kind() else {
                return Err(contract(
                    "Core list result does not declare list type",
                    instruction,
                ));
            };
            Value::list(element.clone(), values)
        }
        .map_err(|_| contract("Core sequence construction failed", instruction))?;
        Ok(RuntimeValue::Public(value))
    }

    pub(super) fn map_begin(
        &self,
        entries: u32,
        instruction: &CoreInstruction,
        program: &CoreProgram,
    ) -> Result<RuntimeValue, ExpressionError> {
        let map_type = internal_map_type(program, instruction)?;
        let count =
            usize::try_from(entries).map_err(|_| contract("map size overflows", instruction))?;
        self.execution
            .reserve_map_collection(count, instruction.span.clone())?;
        Ok(RuntimeValue::MapBuilder(MapBuilderState {
            map_type,
            expected: entries,
            next: 0,
            entries: Vec::with_capacity(count),
            keys: BTreeSet::new(),
        }))
    }

    pub(super) fn map_key(
        &self,
        builder: crate::program::expression::ValueId,
        key: crate::program::expression::ValueId,
        ordinal: u32,
        instruction: &CoreInstruction,
        values: &mut [Option<RuntimeValue>],
    ) -> Result<RuntimeValue, ExpressionError> {
        let key = public(values, key, instruction)?;
        let mut builder = take_builder(values, builder, instruction)?;
        if ordinal != builder.next || ordinal >= builder.expected {
            return Err(contract("Core map ordinal is invalid", instruction));
        }
        if !builder.keys.insert(map_key_identity(&key, instruction)?) {
            return Err(ExpressionError::new(
                "EXPRESSION_DUPLICATE_MAP_KEY",
                "map contains a duplicate key",
                instruction.span.clone(),
            ));
        }
        builder.next = builder
            .next
            .checked_add(1)
            .ok_or_else(|| contract("Core map ordinal overflows", instruction))?;
        Ok(RuntimeValue::MapPending(MapPendingState { builder, key }))
    }

    pub(super) fn map_value(
        &self,
        pending: crate::program::expression::ValueId,
        value: crate::program::expression::ValueId,
        instruction: &CoreInstruction,
        program: &CoreProgram,
        values: &mut [Option<RuntimeValue>],
    ) -> Result<RuntimeValue, ExpressionError> {
        let value = public(values, value, instruction)?;
        let mut pending = take_pending(values, pending, instruction)?;
        require_map_value_type(&pending.builder, &value, instruction, program)?;
        pending
            .builder
            .entries
            .push(MapValueEntry::new(pending.key, value));
        Ok(RuntimeValue::MapBuilder(pending.builder))
    }

    pub(super) fn map_finish(
        &self,
        builder: crate::program::expression::ValueId,
        instruction: &CoreInstruction,
        program: &CoreProgram,
        values: &mut [Option<RuntimeValue>],
    ) -> Result<RuntimeValue, ExpressionError> {
        let builder = take_builder(values, builder, instruction)?;
        if builder.next != builder.expected || builder.entries.len() != builder.expected as usize {
            return Err(contract("Core map entry count is invalid", instruction));
        }
        let expected = program
            .value_type(builder.map_type)
            .ok_or_else(|| contract("Core map type is unavailable", instruction))?;
        let ValueTypeKind::Map { key, value } = expected.kind() else {
            return Err(contract(
                "Core map token does not reference map type",
                instruction,
            ));
        };
        let map = MapValue::new(key, value.clone(), builder.entries)
            .map_err(|_| contract("Core map construction failed", instruction))?;
        Ok(RuntimeValue::Public(Value::Map(Arc::new(map))))
    }
}

fn internal_map_type(
    program: &CoreProgram,
    instruction: &CoreInstruction,
) -> Result<CoreTypeId, ExpressionError> {
    match program.types.get(instruction.type_id) {
        Some(CoreType::MapBuilder { map_type }) => Ok(*map_type),
        _ => Err(contract(
            "Core map builder type is unavailable",
            instruction,
        )),
    }
}

fn require_map_value_type(
    builder: &MapBuilderState,
    value: &Value,
    instruction: &CoreInstruction,
    program: &CoreProgram,
) -> Result<(), ExpressionError> {
    let Some(expected) = program.value_type(builder.map_type) else {
        return Err(contract("Core map type is unavailable", instruction));
    };
    matches!(expected.kind(), ValueTypeKind::Map { value: expected, .. } if expected == &value.value_type())
        .then_some(())
        .ok_or_else(|| contract("Core map value has the wrong type", instruction))
}

fn map_key_identity(
    value: &Value,
    instruction: &CoreInstruction,
) -> Result<MapKeyIdentity, ExpressionError> {
    match value {
        Value::Text(value) => Ok(MapKeyIdentity::Text(Arc::clone(value))),
        Value::Identifier(value) => Ok(MapKeyIdentity::Identifier(Arc::clone(value))),
        _ => Err(contract("Core map key has the wrong type", instruction)),
    }
}
