use super::value::contract;
use crate::program::expression::core::{
    CoreInstruction, CoreProgram, CoreType, CoreTypeId, ValueId,
};
use crate::program::expression::{ExpressionError, MapValueEntry, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

pub(super) enum RuntimeValue {
    Public(Value),
    MapBuilder(MapBuilderState),
    MapPending(MapPendingState),
}

pub(super) struct MapBuilderState {
    pub(super) map_type: CoreTypeId,
    pub(super) expected: u32,
    pub(super) next: u32,
    pub(super) entries: Vec<MapValueEntry>,
    pub(super) keys: BTreeSet<MapKeyIdentity>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum MapKeyIdentity {
    Text(Arc<str>),
    Identifier(Arc<str>),
}

pub(super) struct MapPendingState {
    pub(super) builder: MapBuilderState,
    pub(super) key: Value,
}

pub(super) fn public(
    values: &[Option<RuntimeValue>],
    id: ValueId,
    instruction: &CoreInstruction,
) -> Result<Value, ExpressionError> {
    public_at(values, id, &instruction.span)
}

pub(super) fn public_at(
    values: &[Option<RuntimeValue>],
    id: ValueId,
    span: &std::ops::Range<usize>,
) -> Result<Value, ExpressionError> {
    match values.get(id.index().unwrap_or(usize::MAX)) {
        Some(Some(RuntimeValue::Public(value))) => Ok(value.clone()),
        _ => Err(ExpressionError::new(
            "EXPRESSION_RUNTIME_CONTRACT",
            "Core public value is unavailable",
            span.clone(),
        )),
    }
}

pub(super) fn public_ref<'a>(
    values: &'a [Option<RuntimeValue>],
    id: ValueId,
    instruction: &CoreInstruction,
) -> Result<&'a Value, ExpressionError> {
    match values.get(id.index().unwrap_or(usize::MAX)) {
        Some(Some(RuntimeValue::Public(value))) => Ok(value),
        _ => Err(contract("Core public value is unavailable", instruction)),
    }
}

pub(super) fn take_builder(
    values: &mut [Option<RuntimeValue>],
    id: ValueId,
    instruction: &CoreInstruction,
) -> Result<MapBuilderState, ExpressionError> {
    match take(values, id) {
        Some(RuntimeValue::MapBuilder(value)) => Ok(value),
        _ => Err(contract("Core map builder is unavailable", instruction)),
    }
}

pub(super) fn take_pending(
    values: &mut [Option<RuntimeValue>],
    id: ValueId,
    instruction: &CoreInstruction,
) -> Result<MapPendingState, ExpressionError> {
    match take(values, id) {
        Some(RuntimeValue::MapPending(value)) => Ok(value),
        _ => Err(contract(
            "Core pending map entry is unavailable",
            instruction,
        )),
    }
}

pub(super) fn verify_contract(
    value: &RuntimeValue,
    instruction: &CoreInstruction,
    program: &CoreProgram,
) -> Result<(), ExpressionError> {
    let valid = match (program.types.get(instruction.type_id), value) {
        (Some(CoreType::Value(expected)), RuntimeValue::Public(value)) => {
            expected == &value.value_type()
        }
        (Some(CoreType::MapBuilder { map_type }), RuntimeValue::MapBuilder(value)) => {
            map_type == &value.map_type
        }
        (Some(CoreType::MapPending { map_type }), RuntimeValue::MapPending(value)) => {
            map_type == &value.builder.map_type
        }
        _ => false,
    };
    valid.then_some(()).ok_or_else(|| {
        contract(
            "compiled instruction produced the wrong runtime type",
            instruction,
        )
    })
}

fn take(values: &mut [Option<RuntimeValue>], id: ValueId) -> Option<RuntimeValue> {
    values
        .get_mut(id.index().unwrap_or(usize::MAX))
        .and_then(Option::take)
}

#[cfg(test)]
#[path = "slot/tests.rs"]
mod tests;
