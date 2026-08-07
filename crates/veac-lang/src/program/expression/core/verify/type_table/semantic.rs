use super::super::error;
use crate::program::expression::core::{CoreType, CoreTypeId, CoreTypeTable};
use crate::program::expression::{
    ExpressionError, ValueType, ValueTypeKind, MAX_VALUE_TYPE_ARITY, MAX_VALUE_TYPE_DEPTH,
};

pub(super) fn verify_entry(
    table: &CoreTypeTable,
    index: usize,
    kind: &CoreType,
) -> Result<(), ExpressionError> {
    match kind {
        CoreType::Value(value) => {
            if value.depth() > MAX_VALUE_TYPE_DEPTH {
                return Err(error("Core value type exceeds its depth limit", 0..0));
            }
            for child in children(value)? {
                require_earlier_value(table, index, &child)?;
            }
        }
        CoreType::MapBuilder { map_type } | CoreType::MapPending { map_type } => {
            let Some(CoreType::Value(value)) = earlier(table, index, *map_type) else {
                return Err(error(
                    "Core map token requires an earlier map value type",
                    0..0,
                ));
            };
            if !matches!(value.kind(), ValueTypeKind::Map { .. }) {
                return Err(error("Core map token must reference a map type", 0..0));
            }
        }
    }
    Ok(())
}

pub(super) fn child_ids(
    table: &CoreTypeTable,
    index: usize,
) -> Result<Vec<CoreTypeId>, ExpressionError> {
    match &table.entries()[index].kind {
        CoreType::Value(value) => children(value)?
            .iter()
            .map(|child| require_earlier_value(table, index, child))
            .collect(),
        CoreType::MapBuilder { map_type } | CoreType::MapPending { map_type } => {
            Ok(vec![*map_type])
        }
    }
}

fn children(value: &ValueType) -> Result<Vec<ValueType>, ExpressionError> {
    let children = match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) | ValueTypeKind::Nominal(_) => {
            Vec::new()
        }
        ValueTypeKind::List(element) | ValueTypeKind::Range(element) => vec![element.clone()],
        ValueTypeKind::Map { key, value } => {
            vec![ValueType::primitive(key.primitive()), value.clone()]
        }
        ValueTypeKind::Tuple(elements) => {
            require_arity(elements.len(), 2)?;
            elements.to_vec()
        }
        ValueTypeKind::Function {
            parameters, result, ..
        } => {
            require_arity(parameters.len(), 0)?;
            parameters.iter().cloned().chain([result.clone()]).collect()
        }
    };
    Ok(children)
}

fn require_arity(actual: usize, minimum: usize) -> Result<(), ExpressionError> {
    if actual < minimum || actual > MAX_VALUE_TYPE_ARITY {
        Err(error("Core value type has invalid arity", 0..0))
    } else {
        Ok(())
    }
}

fn require_earlier_value(
    table: &CoreTypeTable,
    parent: usize,
    expected: &ValueType,
) -> Result<CoreTypeId, ExpressionError> {
    table.entries()[..parent]
        .iter()
        .find(|entry| entry.kind().value_type() == Some(expected))
        .map(|entry| entry.id())
        .ok_or_else(|| error("Core semantic child type must precede its parent", 0..0))
}

fn earlier(table: &CoreTypeTable, parent: usize, id: CoreTypeId) -> Option<&CoreType> {
    id.index()
        .filter(|index| *index < parent)
        .and_then(|_| table.get(id))
}
