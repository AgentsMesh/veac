use std::collections::BTreeSet;
use std::ops::Range;

use crate::program::expression::{ExpressionError, Value};
use crate::program::{
    DomainInstructionKind, DomainOperationContract, DomainValueShape, OperandAxis,
};

use super::super::super::provenance::DomainProvenance;
use super::super::super::state::{NodeAllocation, NodeId, RelationReferences};
use super::super::super::{accounting, error, validation, DomainGraphTransaction};
use super::super::support::{identifier, logical_overflow, require_kind, result_type};

pub(in crate::program::expression::runtime::domain_graph::operation) fn relation(
    transaction: &mut DomainGraphTransaction<'_>,
    contract: &DomainOperationContract,
    operands: Vec<Value>,
    bytes: usize,
    provenance: Option<DomainProvenance>,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    require_kind(contract, DomainInstructionKind::GraphEmit, &span)?;
    let key = identifier(operands.first(), &span)?;
    let references = references(transaction, contract, &operands, &span)?;
    let bytes = bytes
        .checked_add(
            accounting::reference_bytes(references.len()).ok_or_else(|| logical_overflow(&span))?,
        )
        .ok_or_else(|| logical_overflow(&span))?;
    transaction.execution.reserve_domain_graph(1, bytes, span)?;
    let allocation = NodeAllocation::relation(
        result_type(contract),
        key,
        RelationReferences::new(references),
        bytes,
    );
    Ok(transaction.state.insert(
        &transaction.scope,
        contract.id(),
        operands,
        allocation,
        provenance,
    ))
}

fn references(
    transaction: &DomainGraphTransaction<'_>,
    contract: &DomainOperationContract,
    operands: &[Value],
    span: &Range<usize>,
) -> Result<Vec<NodeId>, ExpressionError> {
    let mut references = Vec::new();
    for (operand, value) in contract.operands().iter().zip(operands) {
        if operand.axis() != OperandAxis::Topology {
            continue;
        }
        match (operand.shape(), value) {
            (DomainValueShape::Domain(kind), value) if kind.is_graph_entity() => {
                references.push(validation::node(transaction, value, kind, span)?);
            }
            (DomainValueShape::DomainList(kind), Value::List(values)) if kind.is_graph_entity() => {
                for value in values.values() {
                    references.push(validation::node(transaction, value, kind, span)?);
                }
            }
            _ => {}
        }
    }
    let unique = references.iter().copied().collect::<BTreeSet<_>>();
    if references.is_empty() || unique.len() != references.len() {
        return Err(error(
            "DOMAIN_RELATION_REFERENCES",
            "a relation requires a non-empty unique set of typed graph references",
            span.clone(),
        ));
    }
    Ok(references)
}
