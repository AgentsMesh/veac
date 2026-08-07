use crate::program::expression::{ExactNumber, Value};
use crate::program::{DomainOperationId, DomainType};

use super::error::ExecutableLowerError;
use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};

pub(super) fn entity_operands(
    entity: FrozenEntity<'_>,
    expected_type: DomainType,
    expected_operation: DomainOperationId,
) -> Result<&[Value], ExecutableLowerError> {
    if entity.domain_type() != Some(expected_type) {
        return Err(graph("an executable graph entity has an unexpected type"));
    }
    let (operation, operands) = entity
        .constructor()
        .ok_or_else(|| graph("an executable graph entity has no constructor"))?;
    (operation == expected_operation)
        .then_some(operands)
        .ok_or_else(|| graph("an executable graph entity has an unexpected constructor"))
}

pub(super) fn description_operands<'a>(
    graph: &'a FrozenDomainGraph,
    value: &Value,
    expected: DomainOperationId,
) -> Result<&'a [Value], ExecutableLowerError> {
    let Value::Domain(handle) = value else {
        return Err(graph_error());
    };
    if graph.operation(handle) != Some(expected) {
        return Err(graph_error());
    }
    graph.operands(handle).ok_or_else(graph_error)
}

pub(super) fn description<'a>(
    graph: &'a FrozenDomainGraph,
    value: &Value,
) -> Result<(DomainOperationId, &'a [Value]), ExecutableLowerError> {
    let Value::Domain(handle) = value else {
        return Err(graph_error());
    };
    let operation = graph.operation(handle).ok_or_else(graph_error)?;
    let operands = graph.operands(handle).ok_or_else(graph_error)?;
    Ok((operation, operands))
}

pub(super) fn exact(value: Option<&Value>) -> Result<ExactNumber, ExecutableLowerError> {
    value
        .and_then(|value| match value {
            Value::Integer(value) => Some(ExactNumber::integer(i128::from(*value))),
            Value::Scalar(value)
            | Value::Time(value)
            | Value::Length(value)
            | Value::Percent(value)
            | Value::Angle(value) => Some(*value),
            _ => None,
        })
        .ok_or_else(graph_error)
}

pub(super) fn integer(value: Option<&Value>) -> Result<i64, ExecutableLowerError> {
    match value {
        Some(Value::Integer(value)) => Ok(*value),
        _ => Err(graph_error()),
    }
}

pub(super) fn boolean(value: Option<&Value>) -> Result<bool, ExecutableLowerError> {
    match value {
        Some(Value::Bool(value)) => Ok(*value),
        _ => Err(graph_error()),
    }
}

pub(super) fn identifier(value: Option<&Value>) -> Result<&str, ExecutableLowerError> {
    match value {
        Some(Value::Identifier(value)) => Ok(value),
        _ => Err(graph_error()),
    }
}

pub(super) fn list(value: Option<&Value>) -> Result<&[Value], ExecutableLowerError> {
    match value {
        Some(Value::List(value)) => Ok(value.values()),
        _ => Err(graph_error()),
    }
}

pub(super) fn finite(value: Option<&Value>) -> Result<f64, ExecutableLowerError> {
    let exact = exact(value)?;
    safe_f64(exact).ok_or_else(graph_error)
}

pub(super) fn safe_f64(exact: ExactNumber) -> Option<f64> {
    const MAX_SAFE_INTEGER: u128 = 9_007_199_254_740_991;
    if exact.numerator().unsigned_abs() > MAX_SAFE_INTEGER
        || exact.denominator() as u128 > MAX_SAFE_INTEGER
    {
        return None;
    }
    let result = exact.numerator() as f64 / exact.denominator() as f64;
    result.is_finite().then_some(result)
}

pub(super) fn text(value: Option<&Value>) -> Result<&str, ExecutableLowerError> {
    match value {
        Some(Value::Text(value)) => Ok(value),
        _ => Err(graph_error()),
    }
}

pub(super) fn color(value: Option<&Value>) -> Result<veac_ir::Color, ExecutableLowerError> {
    let Some(Value::Color(value)) = value else {
        return Err(color_error());
    };
    let Some(digits) = value.strip_prefix('#') else {
        return Err(color_error());
    };
    let digits = digits.as_bytes();
    if !digits.iter().all(u8::is_ascii_hexdigit) {
        return Err(color_error());
    }
    let alpha = match digits.len() {
        6 => u8::MAX,
        8 => component(&digits[6..8]),
        _ => return Err(color_error()),
    };
    Ok(veac_ir::Color {
        red: component(&digits[0..2]),
        green: component(&digits[2..4]),
        blue: component(&digits[4..6]),
        alpha,
    })
}

fn component(value: &[u8]) -> u8 {
    nibble(value[0]) * 16 + nibble(value[1])
}

fn nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => 0,
    }
}

pub(super) fn graph(message: &'static str) -> ExecutableLowerError {
    ExecutableLowerError::lower("EXECUTABLE_LOWER_GRAPH", message)
}

fn graph_error() -> ExecutableLowerError {
    graph("an executable graph constructor has invalid operands")
}

fn color_error() -> ExecutableLowerError {
    ExecutableLowerError::lower(
        "EXECUTABLE_LOWER_COLOR",
        "an executable color must use #RRGGBB or #RRGGBBAA",
    )
}
