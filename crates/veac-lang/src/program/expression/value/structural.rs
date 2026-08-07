use std::sync::Arc;

use super::Value;
use crate::program::expression::execution_budget::LOGICAL_COLLECTION_HANDLE_BYTES;
use crate::program::expression::{MapKeyType, ValueType};

mod error;
pub use error::ValueConstructionError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListValue {
    value_type: ValueType,
    values: Arc<[Value]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TupleValue {
    value_type: ValueType,
    values: Arc<[Value]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapValue {
    value_type: ValueType,
    entries: Arc<[MapValueEntry]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapValueEntry {
    key: Value,
    value: Value,
}

impl ListValue {
    pub(crate) fn new(
        element: ValueType,
        values: Vec<Value>,
    ) -> Result<Self, ValueConstructionError> {
        if values.iter().any(|value| value.value_type() != element) {
            return Err(ValueConstructionError::new(
                "VALUE_LIST_ELEMENT_TYPE",
                "list element type does not match its resolved type",
            ));
        }
        super::domain_affinity::validate(&values)?;
        Ok(Self {
            value_type: ValueType::list(element).map_err(ValueConstructionError::value_type)?,
            values: values.into(),
        })
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        retained_values(&self.values)
    }
}

impl TupleValue {
    pub(crate) fn new(values: Vec<Value>) -> Result<Self, ValueConstructionError> {
        super::domain_affinity::validate(&values)?;
        let types = values.iter().map(Value::value_type).collect();
        Ok(Self {
            value_type: ValueType::tuple(types).map_err(ValueConstructionError::value_type)?,
            values: values.into(),
        })
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        retained_values(&self.values)
    }
}

impl MapValue {
    pub(crate) fn new(
        key_type: MapKeyType,
        value_type: ValueType,
        mut entries: Vec<MapValueEntry>,
    ) -> Result<Self, ValueConstructionError> {
        if entries.iter().any(|entry| {
            entry.key.primitive_kind() != Some(key_type.primitive())
                || entry.value.value_type() != value_type
        }) {
            return Err(ValueConstructionError::new(
                "VALUE_MAP_ENTRY_TYPE",
                "map entry does not match its resolved type",
            ));
        }
        super::domain_affinity::validate(
            entries.iter().flat_map(|entry| [&entry.key, &entry.value]),
        )?;
        entries.sort_by(|left, right| key_bytes(&left.key).cmp(key_bytes(&right.key)));
        if entries
            .windows(2)
            .any(|values| key_bytes(&values[0].key) == key_bytes(&values[1].key))
        {
            return Err(ValueConstructionError::new(
                "VALUE_MAP_DUPLICATE_KEY",
                "map contains a duplicate key",
            ));
        }
        let key = ValueType::primitive(key_type.primitive());
        Ok(Self {
            value_type: ValueType::map(key, value_type)
                .map_err(ValueConstructionError::value_type)?,
            entries: entries.into(),
        })
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub fn entries(&self) -> &[MapValueEntry] {
        &self.entries
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        self.entries.iter().fold(0usize, |bytes, entry| {
            bytes
                .saturating_add(2 * LOGICAL_COLLECTION_HANDLE_BYTES)
                .saturating_add(entry.key.retained_bytes())
                .saturating_add(entry.value.retained_bytes())
        })
    }
}

impl MapValueEntry {
    pub(crate) fn new(key: Value, value: Value) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &Value {
        &self.key
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

fn key_bytes(value: &Value) -> &[u8] {
    match value {
        Value::Text(value) => value.as_bytes(),
        Value::Identifier(value) => value.as_bytes(),
        _ => unreachable!("map construction validates its key type"),
    }
}

fn retained_values(values: &[Value]) -> usize {
    values.iter().fold(0usize, |bytes, value| {
        bytes
            .saturating_add(LOGICAL_COLLECTION_HANDLE_BYTES)
            .saturating_add(value.retained_bytes())
    })
}
