use super::super::evaluator::Evaluator;
use super::super::map_state::{MapBuilder, MapPending, MapState};
use super::super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{
    CoreInstruction, CoreType, CoreTypeId, Value, ValueId, ValueTypeKind,
};

impl Evaluator<'_> {
    pub(super) fn map_begin(
        &mut self,
        entries: u32,
        instruction: &CoreInstruction,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let count =
            usize::try_from(entries).map_err(|_| contract("map size overflows", instruction))?;
        self.concrete_budget
            .reserve_map_collection(count, instruction.span())
            .map_err(ResidualizationError::expression)?;
        let map_type = internal_type(self, instruction)?;
        self.put(
            instruction.id(),
            MapState::Builder(MapBuilder {
                map_type,
                expected: entries,
                next: 0,
                entries: Vec::with_capacity(count),
                keys: Vec::with_capacity(count),
            }),
        );
        Ok(token())
    }

    pub(super) fn map_key(
        &mut self,
        builder: ValueId,
        key: ValueId,
        ordinal: u32,
        instruction: &CoreInstruction,
        slots: &[Option<ResidualRuntimeValue>],
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let key = concrete(self.value(slots, key, instruction.span())?, instruction)?;
        if !matches!(key, Value::Text(_) | Value::Identifier(_)) {
            return Err(contract("map key has the wrong type", instruction));
        }
        let MapState::Builder(mut builder) = self.take(builder, instruction)? else {
            return Err(contract("map builder token is invalid", instruction));
        };
        if ordinal != builder.next || ordinal >= builder.expected {
            return Err(contract("map ordinal is invalid", instruction));
        }
        if builder.keys.contains(&key) {
            return Err(ResidualizationError::new(
                "EXPRESSION_DUPLICATE_MAP_KEY",
                "map contains a duplicate key",
                instruction.span(),
            ));
        }
        builder.next += 1;
        builder.keys.push(key.clone());
        self.put(
            instruction.id(),
            MapState::Pending(MapPending { builder, key }),
        );
        Ok(token())
    }

    pub(super) fn map_value(
        &mut self,
        pending: ValueId,
        value: ValueId,
        instruction: &CoreInstruction,
        slots: &[Option<ResidualRuntimeValue>],
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let value = concrete(self.value(slots, value, instruction.span())?, instruction)?;
        let MapState::Pending(mut pending) = self.take(pending, instruction)? else {
            return Err(contract("map pending token is invalid", instruction));
        };
        require_value_type(self, pending.builder.map_type, &value, instruction)?;
        pending.builder.entries.push((pending.key, value));
        self.put(instruction.id(), MapState::Builder(pending.builder));
        Ok(token())
    }

    pub(super) fn map_finish(
        &mut self,
        builder: ValueId,
        instruction: &CoreInstruction,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let MapState::Builder(builder) = self.take(builder, instruction)? else {
            return Err(contract("map builder token is invalid", instruction));
        };
        if builder.next != builder.expected || builder.entries.len() != builder.expected as usize {
            return Err(contract("map entry count is invalid", instruction));
        }
        let Some(value_type) = self.core().value_type(builder.map_type) else {
            return Err(contract("map type is unavailable", instruction));
        };
        let ValueTypeKind::Map { key, value } = value_type.kind() else {
            return Err(contract(
                "map token does not reference a map type",
                instruction,
            ));
        };
        Value::map(key, value.clone(), builder.entries)
            .map(ResidualRuntimeValue::Concrete)
            .map_err(|_| contract("map construction failed", instruction))
    }

    fn put(&mut self, id: ValueId, state: MapState) {
        self.map_states.insert((self.call_depth, id), state);
    }

    fn take(
        &mut self,
        id: ValueId,
        instruction: &CoreInstruction,
    ) -> Result<MapState, ResidualizationError> {
        self.map_states
            .remove(&(self.call_depth, id))
            .ok_or_else(|| contract("map token is unavailable", instruction))
    }
}

fn internal_type(
    evaluator: &Evaluator<'_>,
    instruction: &CoreInstruction,
) -> Result<CoreTypeId, ResidualizationError> {
    match evaluator.core().types().get(instruction.type_id()) {
        Some(CoreType::MapBuilder { map_type }) => Ok(*map_type),
        _ => Err(contract("map builder type is unavailable", instruction)),
    }
}

fn require_value_type(
    evaluator: &Evaluator<'_>,
    map_type: CoreTypeId,
    value: &Value,
    instruction: &CoreInstruction,
) -> Result<(), ResidualizationError> {
    let expected = evaluator.core().value_type(map_type);
    matches!(expected.map(|value| value.kind()), Some(ValueTypeKind::Map { value: expected, .. }) if expected == &value.value_type())
        .then_some(())
        .ok_or_else(|| contract("map value has the wrong type", instruction))
}

fn concrete(
    value: ResidualRuntimeValue,
    instruction: &CoreInstruction,
) -> Result<Value, ResidualizationError> {
    match value {
        ResidualRuntimeValue::Concrete(value) => Ok(value),
        _ => Err(ResidualizationError::new(
            "RESIDUAL_VALUE_UNSUPPORTED",
            "map entries must be known at Build stage",
            instruction.span(),
        )),
    }
}

fn token() -> ResidualRuntimeValue {
    ResidualRuntimeValue::Concrete(Value::Integer(0))
}

fn contract(message: &str, instruction: &CoreInstruction) -> ResidualizationError {
    ResidualizationError::new("RESIDUAL_CORE_CONTRACT", message, instruction.span())
}
