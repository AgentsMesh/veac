use std::ops::Range;
use std::sync::Arc;

use crate::program::expression::{ExpressionError, Value};
use crate::program::{
    DomainInstructionKind, DomainOperationContract, DomainRuntimeAction, DomainType,
};

use super::super::provenance::ProvenanceKind;
use super::super::state::NodeId;
use super::super::{error, validation, DomainGraphTransaction};

pub(super) fn child_nodes(
    transaction: &DomainGraphTransaction<'_>,
    value: &Value,
    contract: &DomainOperationContract,
    span: &Range<usize>,
) -> Result<Vec<NodeId>, ExpressionError> {
    let expected = contract.operands()[1]
        .shape()
        .domain_type()
        .expect("attachment child is a domain shape");
    match value {
        Value::List(values) => values
            .values()
            .iter()
            .map(|value| validation::node(transaction, value, expected, span))
            .collect(),
        value => Ok(vec![validation::node(transaction, value, expected, span)?]),
    }
}

pub(super) fn identifier(
    value: Option<&Value>,
    span: &Range<usize>,
) -> Result<Arc<str>, ExpressionError> {
    match value {
        Some(Value::Identifier(value)) if !value.is_empty() => Ok(value.clone()),
        _ => Err(error(
            "DOMAIN_KEY_REQUIRED",
            "graph entities require a non-empty explicit identifier key",
            span.clone(),
        )),
    }
}

pub(super) fn result_type(contract: &DomainOperationContract) -> DomainType {
    contract
        .result()
        .domain_type()
        .expect("verified domain result")
}

pub(super) fn require_kind(
    contract: &DomainOperationContract,
    expected: DomainInstructionKind,
    span: &Range<usize>,
) -> Result<(), ExpressionError> {
    (contract.instruction() == expected)
        .then_some(())
        .ok_or_else(|| {
            error(
                "DOMAIN_OPERATION_CONTRACT",
                "domain operation instruction kind violates its verified contract",
                span.clone(),
            )
        })
}

pub(super) const fn kind(action: DomainRuntimeAction) -> ProvenanceKind {
    match action {
        DomainRuntimeAction::Description => ProvenanceKind::Construct,
        DomainRuntimeAction::EntityConstructor | DomainRuntimeAction::RelationConstructor => {
            ProvenanceKind::Constructor
        }
        DomainRuntimeAction::OwnedAttachment
        | DomainRuntimeAction::NonOwningUpdate
        | DomainRuntimeAction::ProjectEntry => ProvenanceKind::Update,
    }
}

pub(super) fn logical_overflow(span: &Range<usize>) -> ExpressionError {
    error(
        "DOMAIN_LOGICAL_BYTES_OVERFLOW",
        "domain graph logical storage overflows",
        span.clone(),
    )
}
