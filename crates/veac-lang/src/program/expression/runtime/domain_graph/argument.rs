use std::ops::Range;

use super::super::super::{ExpressionError, Value};
use super::DomainGraphTransaction;

impl DomainGraphTransaction<'_> {
    pub(in crate::program::expression) fn validate_argument(
        &self,
        value: &Value,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        validate(self, value, &span)
    }
}

fn validate(
    transaction: &DomainGraphTransaction<'_>,
    value: &Value,
    span: &Range<usize>,
) -> Result<(), ExpressionError> {
    match value {
        Value::Domain(handle) => {
            super::validation::node(transaction, value, handle.domain_type(), span).map(drop)
        }
        Value::List(value) => sequence(transaction, value.values(), span),
        Value::Tuple(value) => sequence(transaction, value.values(), span),
        Value::Struct(value) => sequence(transaction, value.fields(), span),
        Value::Enum(value) => sequence(transaction, value.fields(), span),
        Value::Closure(value) => sequence(transaction, value.captures(), span),
        Value::Map(value) => value.entries().iter().try_for_each(|entry| {
            validate(transaction, entry.key(), span)?;
            validate(transaction, entry.value(), span)
        }),
        _ => Ok(()),
    }
}

fn sequence(
    transaction: &DomainGraphTransaction<'_>,
    values: &[Value],
    span: &Range<usize>,
) -> Result<(), ExpressionError> {
    values
        .iter()
        .try_for_each(|value| validate(transaction, value, span))
}
