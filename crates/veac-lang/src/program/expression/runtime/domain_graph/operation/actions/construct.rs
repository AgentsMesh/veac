use std::ops::Range;

use crate::program::expression::{ExpressionError, Value};
use crate::program::{DomainInstructionKind, DomainOperationContract};

use super::super::super::provenance::DomainProvenance;
use super::super::super::state::NodeAllocation;
use super::super::super::DomainGraphTransaction;
use super::super::support::{identifier, require_kind, result_type};

pub(in crate::program::expression::runtime::domain_graph::operation) fn construct(
    transaction: &mut DomainGraphTransaction<'_>,
    contract: &DomainOperationContract,
    operands: Vec<Value>,
    bytes: usize,
    provenance: Option<DomainProvenance>,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    require_kind(contract, DomainInstructionKind::DomainConstruct, &span)?;
    transaction.execution.reserve_domain_graph(0, bytes, span)?;
    let allocation = NodeAllocation::description(result_type(contract), bytes);
    Ok(transaction.state.insert(
        &transaction.scope,
        contract.id(),
        operands,
        allocation,
        provenance,
    ))
}

pub(in crate::program::expression::runtime::domain_graph::operation) fn entity(
    transaction: &mut DomainGraphTransaction<'_>,
    contract: &DomainOperationContract,
    operands: Vec<Value>,
    bytes: usize,
    provenance: Option<DomainProvenance>,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    require_kind(contract, DomainInstructionKind::GraphEmit, &span)?;
    let key = identifier(operands.first(), &span)?;
    transaction.execution.reserve_domain_graph(1, bytes, span)?;
    let allocation = NodeAllocation::entity(result_type(contract), key, bytes);
    Ok(transaction.state.insert(
        &transaction.scope,
        contract.id(),
        operands,
        allocation,
        provenance,
    ))
}
