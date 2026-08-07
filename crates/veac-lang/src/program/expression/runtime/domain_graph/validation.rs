use std::ops::Range;

use crate::program::{DomainOperationContract, DomainType, DomainValueShape};

use super::super::super::{ExpressionError, PrimitiveType, Value};
use super::state::NodeId;
use super::{error, DomainGraphTransaction};

mod ownership;
mod refinement;
pub(super) use ownership::{attachment, unowned};

pub(super) fn refinement(
    contract: &DomainOperationContract,
    values: &[Value],
    span: &Range<usize>,
) -> Result<(), ExpressionError> {
    refinement::validate(contract, values, span)
}

pub(super) fn operands(
    transaction: &DomainGraphTransaction<'_>,
    contract: &DomainOperationContract,
    values: &[Value],
    span: &Range<usize>,
) -> Result<(), ExpressionError> {
    if contract.operands().len() != values.len() {
        return Err(error(
            "DOMAIN_OPERAND_ARITY",
            format!(
                "operation `{}` expects {} operands, found {}",
                contract.name(),
                contract.operands().len(),
                values.len()
            ),
            span.clone(),
        ));
    }
    contract
        .operands()
        .iter()
        .zip(values)
        .try_for_each(|(operand, value)| {
            shape(transaction, operand.shape(), value, operand.name(), span)
        })
}

pub(super) fn node(
    transaction: &DomainGraphTransaction<'_>,
    value: &Value,
    expected: DomainType,
    span: &Range<usize>,
) -> Result<NodeId, ExpressionError> {
    let Value::Domain(handle) = value else {
        return Err(type_error(expected.name(), value, span));
    };
    if !handle.belongs_to(&transaction.scope) {
        return Err(error(
            "DOMAIN_CROSS_GRAPH",
            "domain handles cannot cross graph transactions",
            span.clone(),
        ));
    }
    let slot = handle.arena_slot() as usize;
    let record = transaction.state.records.get(slot).ok_or_else(|| {
        error(
            "DOMAIN_HANDLE_INVALID",
            "domain handle does not identify an allocated graph value",
            span.clone(),
        )
    })?;
    let stored = &transaction.state.nodes[record.node.0];
    if stored.domain_type != handle.domain_type() {
        return Err(error(
            "DOMAIN_HANDLE_FORGED",
            "domain handle type does not match its arena allocation",
            span.clone(),
        ));
    }
    if stored.domain_type != expected {
        return Err(type_error(expected.name(), value, span));
    }
    if stored.latest_slot as usize != slot {
        return Err(error(
            "DOMAIN_HANDLE_STALE",
            "an immutable graph update must use the returned handle",
            span.clone(),
        ));
    }
    Ok(record.node)
}

fn shape(
    transaction: &DomainGraphTransaction<'_>,
    expected: DomainValueShape,
    value: &Value,
    name: &str,
    span: &Range<usize>,
) -> Result<(), ExpressionError> {
    match expected {
        DomainValueShape::Primitive(expected) if primitive(value) == Some(expected) => Ok(()),
        DomainValueShape::PrimitiveList(expected) => match value {
            Value::List(values) => values.values().iter().try_for_each(|value| {
                (primitive(value) == Some(expected))
                    .then_some(())
                    .ok_or_else(|| type_error(&format!("list<{expected}>"), value, span))
            }),
            _ => Err(type_error(&format!("list<{expected}>"), value, span)),
        },
        DomainValueShape::Domain(expected) => node(transaction, value, expected, span).map(drop),
        DomainValueShape::DomainList(expected) => match value {
            Value::List(values) => values
                .values()
                .iter()
                .try_for_each(|value| node(transaction, value, expected, span).map(drop)),
            _ => Err(type_error(&format!("list<{expected}>"), value, span)),
        },
        _ => Err(error(
            "DOMAIN_OPERAND_TYPE",
            format!(
                "domain operand `{name}` has type {}, which violates its contract",
                value.kind()
            ),
            span.clone(),
        )),
    }
}

fn primitive(value: &Value) -> Option<PrimitiveType> {
    value.primitive_kind()
}

fn type_error(expected: &str, value: &Value, span: &Range<usize>) -> ExpressionError {
    error(
        "DOMAIN_OPERAND_TYPE",
        format!(
            "expected domain operand type {expected}, found {}",
            value.kind()
        ),
        span.clone(),
    )
}
