use super::{budget::Budget, error::TemporalBackendError, value::Expression};

pub(super) fn unary(
    budget: &mut Budget<'_>,
    prefix: &str,
    value: &Expression,
    suffix: &str,
) -> Result<Expression, TemporalBackendError> {
    budget.expression(&[prefix, value, suffix])
}

pub(super) fn infix(
    budget: &mut Budget<'_>,
    left: &Expression,
    operation: &str,
    right: &Expression,
) -> Result<Expression, TemporalBackendError> {
    budget.expression(&["(", left, ")", operation, "(", right, ")"])
}

pub(super) fn call1(
    budget: &mut Budget<'_>,
    function: &str,
    value: &Expression,
) -> Result<Expression, TemporalBackendError> {
    budget.expression(&[function, "(", value, ")"])
}

pub(super) fn call2(
    budget: &mut Budget<'_>,
    function: &str,
    left: &Expression,
    right: &Expression,
) -> Result<Expression, TemporalBackendError> {
    budget.expression(&[function, "(", left, "\\,", right, ")"])
}

pub(super) fn select(
    budget: &mut Budget<'_>,
    condition: &Expression,
    when_true: &Expression,
    when_false: &Expression,
) -> Result<Expression, TemporalBackendError> {
    budget.expression(&["if(", condition, "\\,", when_true, "\\,", when_false, ")"])
}
