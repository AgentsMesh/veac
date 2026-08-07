use std::ops::Range;

use crate::program::expression::{ExpressionError, Value};
use crate::program::{DomainInstructionKind, DomainOperationContract};

use super::super::super::provenance::DomainProvenance;
use super::super::super::{validation, DomainGraphTransaction};
use super::super::support::{require_kind, result_type};

pub(in crate::program::expression::runtime::domain_graph::operation) fn entry(
    transaction: &mut DomainGraphTransaction<'_>,
    contract: &DomainOperationContract,
    operands: Vec<Value>,
    bytes: usize,
    provenance: Option<DomainProvenance>,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    require_kind(contract, DomainInstructionKind::GraphEmit, &span)?;
    let parent = validation::node(transaction, &operands[0], result_type(contract), &span)?;
    let entry_type = contract.operands()[1]
        .shape()
        .domain_type()
        .expect("verified project entry type");
    let entry = validation::node(transaction, &operands[1], entry_type, &span)?;
    validation::unowned(transaction, parent, &span)?;
    transaction.execution.reserve_domain_graph(0, bytes, span)?;
    transaction.state.nodes[parent.0].entry = Some(entry);
    Ok(transaction.state.update(
        &transaction.scope,
        contract.id(),
        operands,
        parent,
        bytes,
        provenance,
    ))
}

pub(in crate::program::expression::runtime::domain_graph::operation) fn update(
    transaction: &mut DomainGraphTransaction<'_>,
    contract: &DomainOperationContract,
    operands: Vec<Value>,
    bytes: usize,
    provenance: Option<DomainProvenance>,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    require_kind(contract, DomainInstructionKind::GraphEmit, &span)?;
    let receiver = contract.operands()[0]
        .shape()
        .domain_type()
        .expect("verified update receiver type");
    let node = validation::node(transaction, &operands[0], receiver, &span)?;
    validation::unowned(transaction, node, &span)?;
    transaction.execution.reserve_domain_graph(0, bytes, span)?;
    Ok(transaction.state.update(
        &transaction.scope,
        contract.id(),
        operands,
        node,
        bytes,
        provenance,
    ))
}
