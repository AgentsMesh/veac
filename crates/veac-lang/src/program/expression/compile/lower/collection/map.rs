use super::{context_error, require_type};
use crate::program::expression::ast::{Expression, MapEntry};
use crate::program::expression::compile::lower::Lowerer;
use crate::program::expression::hir::{TypedMapEntry, TypedNode, TypedNodeKind};
use crate::program::expression::{ExpressionError, MapKeyType, ValueType, ValueTypeKind};

impl Lowerer<'_> {
    pub(in crate::program::expression::compile::lower) fn map(
        &mut self,
        entries: &[MapEntry],
        expected: Option<&ValueType>,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let expected = expected.and_then(map_parts);
        if entries.is_empty() {
            let (key, value) = expected.ok_or_else(|| context_error("map", expression))?;
            let value_type = construct_map(&key.primitive().into(), value, expression)?;
            return Ok((TypedNodeKind::Map(Vec::new()), value_type));
        }
        let (key_type, value_type, mut lowered, start) = match expected {
            Some((key, value)) => (
                key.primitive().into(),
                value.clone(),
                Vec::with_capacity(entries.len()),
                0,
            ),
            None => {
                let first = self.map_entry(&entries[0], None, None)?;
                let key = map_key(&first.key)?;
                let value = first.value.value_type.clone();
                (key.primitive().into(), value, vec![first], 1)
            }
        };
        for entry in &entries[start..] {
            lowered.push(self.map_entry(entry, Some(&key_type), Some(&value_type))?);
        }
        let value_type = construct_map(&key_type, &value_type, expression)?;
        Ok((TypedNodeKind::Map(lowered), value_type))
    }

    fn map_entry(
        &mut self,
        entry: &MapEntry,
        key_type: Option<&ValueType>,
        value_type: Option<&ValueType>,
    ) -> Result<TypedMapEntry, ExpressionError> {
        let key = self.lower_context(&entry.key, key_type)?;
        map_key(&key)?;
        if let Some(expected) = key_type {
            require_type(&key, expected, "EXPRESSION_MAP_KEY_TYPE", "map key")?;
        }
        let value = self.lower_context(&entry.value, value_type)?;
        if let Some(expected) = value_type {
            require_type(&value, expected, "EXPRESSION_MAP_VALUE_TYPE", "map value")?;
        }
        Ok(TypedMapEntry { key, value })
    }
}

fn map_parts(value: &ValueType) -> Option<(MapKeyType, &ValueType)> {
    match value.kind() {
        ValueTypeKind::Map { key, value } => Some((key, value)),
        _ => None,
    }
}

fn map_key(value: &TypedNode) -> Result<MapKeyType, ExpressionError> {
    MapKeyType::try_from(&value.value_type).map_err(|_| {
        ExpressionError::new(
            "EXPRESSION_MAP_KEY_TYPE",
            format!(
                "map key must be text or identifier, found {}",
                value.value_type
            ),
            value.span.clone(),
        )
    })
}

fn construct_map(
    key: &ValueType,
    value: &ValueType,
    expression: &Expression,
) -> Result<ValueType, ExpressionError> {
    ValueType::map(key.clone(), value.clone()).map_err(|error| {
        ExpressionError::new("EXPRESSION_TYPE", error.message(), expression.span.clone())
    })
}
