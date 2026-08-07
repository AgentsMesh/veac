use std::ops::Range;

use super::{
    execution_limit_error, resource, ExecutionBudget, ResourceDelta, LOGICAL_COLLECTION_BASE_BYTES,
    LOGICAL_COLLECTION_HANDLE_BYTES, LOGICAL_NOMINAL_BASE_BYTES,
    LOGICAL_NOMINAL_FIELD_HANDLE_BYTES,
};
use crate::program::expression::{ExpressionError, Value};

impl ExecutionBudget {
    pub(in crate::program::expression) fn admit_value(
        &self,
        value: &Value,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        let delta = value_delta(value).ok_or_else(|| {
            let resource = resource::Resource::CollectionBytes;
            execution_limit_error(
                self.counters[resource.index()].limit,
                resource.label(),
                &span,
            )
        })?;
        self.reserve(delta, span)
    }
}

fn value_delta(value: &Value) -> Option<ResourceDelta> {
    let mut delta = ResourceDelta {
        value_bytes: value.evaluated_bytes(),
        ..ResourceDelta::default()
    };
    match value {
        Value::List(value) => {
            add_container(&mut delta, value.values().len(), 1)?;
            add_values(&mut delta, value.values())?;
        }
        Value::Tuple(value) => {
            add_container(&mut delta, value.values().len(), 1)?;
            add_values(&mut delta, value.values())?;
        }
        Value::Map(value) => {
            add_container(&mut delta, value.entries().len(), 2)?;
            for entry in value.entries() {
                add_delta(&mut delta, value_delta(entry.key())?)?;
                add_delta(&mut delta, value_delta(entry.value())?)?;
            }
        }
        Value::Struct(value) => {
            add_nominal(&mut delta, value.fields().len())?;
            add_values(&mut delta, value.fields())?;
        }
        Value::Enum(value) => {
            add_nominal(&mut delta, value.fields().len())?;
            add_values(&mut delta, value.fields())?;
        }
        _ => {}
    }
    Some(delta)
}

fn add_nominal(delta: &mut ResourceDelta, count: usize) -> Option<()> {
    let bytes = count
        .checked_mul(LOGICAL_NOMINAL_FIELD_HANDLE_BYTES)?
        .checked_add(LOGICAL_NOMINAL_BASE_BYTES)?;
    delta.collection_elements = delta.collection_elements.checked_add(count)?;
    delta.collection_bytes = delta.collection_bytes.checked_add(bytes)?;
    Some(())
}

fn add_values(delta: &mut ResourceDelta, values: &[Value]) -> Option<()> {
    for value in values {
        add_delta(delta, value_delta(value)?)?;
    }
    Some(())
}

fn add_container(delta: &mut ResourceDelta, count: usize, slots: usize) -> Option<()> {
    let bytes = count
        .checked_mul(slots)?
        .checked_mul(LOGICAL_COLLECTION_HANDLE_BYTES)?
        .checked_add(LOGICAL_COLLECTION_BASE_BYTES)?;
    delta.collection_elements = delta.collection_elements.checked_add(count)?;
    delta.collection_bytes = delta.collection_bytes.checked_add(bytes)?;
    Some(())
}

fn add_delta(target: &mut ResourceDelta, value: ResourceDelta) -> Option<()> {
    target.value_bytes = target.value_bytes.checked_add(value.value_bytes)?;
    target.collection_elements = target
        .collection_elements
        .checked_add(value.collection_elements)?;
    target.collection_bytes = target
        .collection_bytes
        .checked_add(value.collection_bytes)?;
    Some(())
}

#[cfg(test)]
#[path = "admission/tests.rs"]
mod tests;
