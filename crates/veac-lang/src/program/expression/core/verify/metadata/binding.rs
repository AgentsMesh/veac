use std::ops::Range;

use super::super::error;
use crate::program::expression::{CoreValueMetadata, ExpressionError, Stage, ValueType};

pub(super) fn parameter(
    index: usize,
    types: &[ValueType],
    stages: &[Stage],
    span: Range<usize>,
) -> Result<CoreValueMetadata, ExpressionError> {
    let value_type = types
        .get(index)
        .ok_or_else(|| error(format!("unknown parameter index {index}"), span.clone()))?;
    let stage = stages
        .get(index)
        .ok_or_else(|| error(format!("unknown parameter stage {index}"), span))?;
    Ok(CoreValueMetadata::typed_parameter(
        *stage, index, value_type,
    ))
}

pub(super) fn capture(
    index: usize,
    types: &[ValueType],
    span: Range<usize>,
) -> Result<CoreValueMetadata, ExpressionError> {
    let value_type = types
        .get(index)
        .ok_or_else(|| error(format!("unknown capture index {index}"), span))?;
    Ok(CoreValueMetadata::capture(Stage::Const, index, value_type))
}
