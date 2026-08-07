use std::sync::Arc;

use super::{ExactNumber, ValueType};
mod closure;
pub(crate) mod domain;
mod domain_affinity;
mod domain_policy;
mod nominal;
mod numeric;
mod range;
mod render;
mod structural;

pub use closure::ClosureValue;
pub use domain::DomainValue;
pub use nominal::{EnumValue, StructValue};
pub use range::RangeValue;
pub use structural::{ListValue, MapValue, MapValueEntry, TupleValue, ValueConstructionError};

crate::define_syntax_tokens! {
    array
    pub enum PrimitiveType {
        Integer => "int",
        Scalar => "scalar",
        Time => "time",
        Length => "length",
        Percent => "percent",
        Angle => "angle",
        Text => "text",
        Color => "color",
        Boolean => "bool",
        Identifier => "identifier",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    Scalar(ExactNumber),
    Time(ExactNumber),
    Length(ExactNumber),
    Percent(ExactNumber),
    Angle(ExactNumber),
    Text(Arc<str>),
    Color(Arc<str>),
    Bool(bool),
    Identifier(Arc<str>),
    Range(Arc<RangeValue>),
    Closure(Arc<ClosureValue>),
    List(Arc<ListValue>),
    Map(Arc<MapValue>),
    Tuple(Arc<TupleValue>),
    Struct(Arc<StructValue>),
    Enum(Arc<EnumValue>),
    Domain(Arc<DomainValue>),
}

impl Value {
    pub fn range(start: i64, end: i64, step: i64) -> Result<Self, ValueConstructionError> {
        RangeValue::new(start, end, step).map(|value| Self::Range(Arc::new(value)))
    }

    pub fn list(
        element_type: ValueType,
        values: Vec<Value>,
    ) -> Result<Self, ValueConstructionError> {
        ListValue::new(element_type, values).map(|value| Self::List(Arc::new(value)))
    }

    pub fn tuple(values: Vec<Value>) -> Result<Self, ValueConstructionError> {
        TupleValue::new(values).map(|value| Self::Tuple(Arc::new(value)))
    }

    pub fn map(
        key_type: super::MapKeyType,
        value_type: ValueType,
        entries: Vec<(Value, Value)>,
    ) -> Result<Self, ValueConstructionError> {
        let entries = entries
            .into_iter()
            .map(|(key, value)| MapValueEntry::new(key, value))
            .collect();
        MapValue::new(key_type, value_type, entries).map(|value| Self::Map(Arc::new(value)))
    }

    pub fn structure(
        registry: &crate::program::TypeRegistry,
        type_id: crate::program::TypeId,
        fields: Vec<Value>,
    ) -> Result<Self, ValueConstructionError> {
        StructValue::new(registry, type_id, fields).map(|value| Self::Struct(Arc::new(value)))
    }

    pub fn variant(
        registry: &crate::program::TypeRegistry,
        type_id: crate::program::TypeId,
        variant: crate::program::VariantIndex,
        fields: Vec<Value>,
    ) -> Result<Self, ValueConstructionError> {
        EnumValue::new(registry, type_id, variant, fields).map(|value| Self::Enum(Arc::new(value)))
    }

    pub fn kind(&self) -> ValueType {
        self.value_type()
    }

    pub fn value_type(&self) -> ValueType {
        match self.primitive_kind() {
            Some(value) => ValueType::primitive(value),
            None => match self {
                Self::List(value) => value.value_type().clone(),
                Self::Map(value) => value.value_type().clone(),
                Self::Tuple(value) => value.value_type().clone(),
                Self::Struct(value) => value.value_type(),
                Self::Enum(value) => value.value_type(),
                Self::Range(_) => ValueType::range(ValueType::primitive(PrimitiveType::Integer))
                    .expect("range<int> is a valid value type"),
                Self::Closure(value) => value.value_type().clone(),
                Self::Domain(value) => ValueType::domain(value.domain_type()),
                _ => unreachable!("every non-structural value has a primitive kind"),
            },
        }
    }

    pub fn primitive_kind(&self) -> Option<PrimitiveType> {
        Some(match self {
            Self::Integer(_) => PrimitiveType::Integer,
            Self::Scalar(_) => PrimitiveType::Scalar,
            Self::Time(_) => PrimitiveType::Time,
            Self::Length(_) => PrimitiveType::Length,
            Self::Percent(_) => PrimitiveType::Percent,
            Self::Angle(_) => PrimitiveType::Angle,
            Self::Text(_) => PrimitiveType::Text,
            Self::Color(_) => PrimitiveType::Color,
            Self::Bool(_) => PrimitiveType::Boolean,
            Self::Identifier(_) => PrimitiveType::Identifier,
            Self::Range(_)
            | Self::Closure(_)
            | Self::List(_)
            | Self::Map(_)
            | Self::Tuple(_)
            | Self::Struct(_)
            | Self::Enum(_)
            | Self::Domain(_) => return None,
        })
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        match self {
            Self::Text(value) => value.len(),
            Self::Color(value) | Self::Identifier(value) => value.len(),
            Self::List(value) => value.retained_bytes(),
            Self::Map(value) => value.retained_bytes(),
            Self::Tuple(value) => value.retained_bytes(),
            Self::Struct(value) => value.retained_bytes(),
            Self::Enum(value) => value.retained_bytes(),
            Self::Domain(value) => value.retained_bytes(),
            Self::Range(_) => super::execution_budget::LOGICAL_RANGE_VALUE_BYTES,
            Self::Closure(_) => super::execution_budget::LOGICAL_CLOSURE_VALUE_BYTES,
            _ => 0,
        }
    }

    pub(crate) fn evaluated_bytes(&self) -> usize {
        match self {
            Self::Text(value) => value.len(),
            Self::Color(value) | Self::Identifier(value) => value.len(),
            Self::Range(_) => super::execution_budget::LOGICAL_RANGE_VALUE_BYTES,
            Self::Closure(_) => super::execution_budget::LOGICAL_CLOSURE_VALUE_BYTES,
            Self::Domain(value) => value.retained_bytes(),
            _ => 0,
        }
    }

    pub(crate) fn validate_nominal_registry(
        &self,
        registry: &crate::program::TypeRegistry,
    ) -> Result<(), ValueConstructionError> {
        nominal::validate_registry(self, registry)
    }
}

#[cfg(test)]
#[path = "value/tests.rs"]
mod tests;
