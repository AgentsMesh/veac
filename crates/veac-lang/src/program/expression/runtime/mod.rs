mod compare;
mod compiled;
#[allow(dead_code)]
pub(in crate::program) mod domain_graph;
mod function;
mod operation;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub(super) use compiled::execute;
pub(in crate::program::expression) use function::call_builtin as residual_builtin;

#[allow(dead_code)]
pub(in crate::program) fn execute_project(
    expression: &super::CompiledExpression,
    environment: &dyn super::ValueLookup,
    execution: &super::ExecutionBudget,
) -> Result<domain_graph::FrozenDomainGraph, super::ExpressionError> {
    compiled::execute_project(expression, environment, execution)
}

pub(in crate::program) fn execute_entry(
    entry: &super::CompiledFunction,
    functions: &super::FunctionMap,
    execution: &super::ExecutionBudget,
    identity: super::ProgramIdentity,
    environment: &dyn super::ValueLookup,
) -> Result<domain_graph::FrozenDomainGraph, super::ExpressionError> {
    compiled::execute_entry(entry, functions, execution, identity, environment)
}

use super::core::CoreUnaryOperator;
use super::{ExpressionError, Value, MAX_TEXT_VALUE_BYTES};

fn validate_value(value: Value, span: std::ops::Range<usize>) -> Result<Value, ExpressionError> {
    validate_value_ref(&value, &span)?;
    Ok(value)
}

fn validate_value_ref(value: &Value, span: &std::ops::Range<usize>) -> Result<(), ExpressionError> {
    if matches!(value, Value::Text(text) if text.len() > MAX_TEXT_VALUE_BYTES) {
        return Err(ExpressionError::new(
            "EXPRESSION_TEXT_LIMIT",
            format!("text value exceeds the {MAX_TEXT_VALUE_BYTES} byte limit"),
            span.clone(),
        ));
    }
    match value {
        Value::List(value) => validate_values(value.values(), span),
        Value::Tuple(value) => validate_values(value.values(), span),
        Value::Struct(value) => validate_values(value.fields(), span),
        Value::Enum(value) => validate_values(value.fields(), span),
        Value::Closure(value) => validate_values(value.captures(), span),
        Value::Map(value) => value.entries().iter().try_for_each(|entry| {
            validate_value_ref(entry.key(), span)?;
            validate_value_ref(entry.value(), span)
        }),
        _ => Ok(()),
    }
}

fn validate_values(values: &[Value], span: &std::ops::Range<usize>) -> Result<(), ExpressionError> {
    values
        .iter()
        .try_for_each(|value| validate_value_ref(value, span))
}

fn reject_external_function(
    value: &Value,
    span: &std::ops::Range<usize>,
) -> Result<(), ExpressionError> {
    let contains = match value {
        Value::Closure(_) => true,
        Value::List(value) => value
            .values()
            .iter()
            .any(|value| reject_external_function(value, span).is_err()),
        Value::Tuple(value) => value
            .values()
            .iter()
            .any(|value| reject_external_function(value, span).is_err()),
        Value::Struct(value) => value
            .fields()
            .iter()
            .any(|value| reject_external_function(value, span).is_err()),
        Value::Enum(value) => value
            .fields()
            .iter()
            .any(|value| reject_external_function(value, span).is_err()),
        Value::Map(value) => value.entries().iter().any(|entry| {
            reject_external_function(entry.key(), span).is_err()
                || reject_external_function(entry.value(), span).is_err()
        }),
        _ => false,
    };
    (!contains).then_some(()).ok_or_else(|| {
        ExpressionError::new(
            "EXPRESSION_EXTERNAL_FUNCTION_VALUE",
            "function values cannot be injected through the external environment",
            span.clone(),
        )
    })
}

fn unary(
    operator: CoreUnaryOperator,
    value: Value,
    span: std::ops::Range<usize>,
) -> Result<Value, ExpressionError> {
    match operator {
        CoreUnaryOperator::Not => match value {
            Value::Bool(value) => Ok(Value::Bool(!value)),
            value => Err(ExpressionError::new(
                "EXPRESSION_TYPE",
                format!("logical negation requires bool, found {}", value.kind()),
                span,
            )),
        },
        CoreUnaryOperator::Positive => value.numeric().map(|_| value).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_TYPE",
                "unary sign requires a numeric value",
                span,
            )
        }),
        CoreUnaryOperator::Negative => value
            .numeric()
            .ok_or_else(|| {
                ExpressionError::new(
                    "EXPRESSION_TYPE",
                    "unary sign requires a numeric value",
                    span.clone(),
                )
            })?
            .checked_neg()
            .and_then(|number| Value::from_numeric(value.primitive_kind()?, number))
            .ok_or_else(|| {
                ExpressionError::new(
                    "EXPRESSION_OVERFLOW",
                    "unary negation exceeds the exact arithmetic range",
                    span,
                )
            }),
    }
}

pub(super) fn residual_unary(
    operator: CoreUnaryOperator,
    value: Value,
    span: std::ops::Range<usize>,
) -> Result<Value, ExpressionError> {
    unary(operator, value, span)
}

pub(super) fn residual_arithmetic(
    execution: &super::ExecutionBudget,
    operator: super::ArithmeticOperator,
    left: Value,
    right: Value,
    span: std::ops::Range<usize>,
) -> Result<Value, ExpressionError> {
    operation::apply(execution, operator, left, right, span)
}

pub(super) fn residual_ordering(
    operator: super::ComparisonOperator,
    left: Value,
    right: Value,
    span: std::ops::Range<usize>,
) -> Result<Value, ExpressionError> {
    compare::ordering(operator, left, right, span)
}

pub(super) fn residual_equality(
    operator: super::EqualityOperator,
    left: Value,
    right: Value,
    span: std::ops::Range<usize>,
) -> Result<Value, ExpressionError> {
    compare::equality(operator, left, right, span)
}
