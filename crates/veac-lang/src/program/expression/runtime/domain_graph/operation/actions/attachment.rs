use std::ops::Range;

use crate::program::expression::{ExpressionError, Value};
use crate::program::{DomainInstructionKind, DomainOperationContract};

use super::super::super::provenance::DomainProvenance;
use super::super::super::{accounting, error, validation, DomainGraphTransaction};
use super::super::support::{child_nodes, logical_overflow, require_kind, result_type};

pub(in crate::program::expression::runtime::domain_graph::operation) fn attach(
    transaction: &mut DomainGraphTransaction<'_>,
    contract: &DomainOperationContract,
    operands: Vec<Value>,
    bytes: usize,
    provenance: Option<DomainProvenance>,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    require_kind(contract, DomainInstructionKind::GraphEmit, &span)?;
    let parent = validation::node(transaction, &operands[0], result_type(contract), &span)?;
    let children = child_nodes(transaction, &operands[1], contract, &span)?;
    validation::attachment(transaction, parent, &children, &span)?;
    let parent_key = transaction.state.nodes[parent.0]
        .key
        .as_deref()
        .ok_or_else(|| {
            error(
                "DOMAIN_KEY_REQUIRED",
                "graph parent has no key",
                span.clone(),
            )
        })?;
    let entities = transaction
        .state
        .provenanced_entity_subtree_count(&children)
        .ok_or_else(|| logical_overflow(&span))?;
    let child_type = contract.operands()[1]
        .shape()
        .domain_type()
        .expect("verified attachment child type");
    let existing = transaction.state.nodes[parent.0]
        .children
        .iter()
        .filter(|child| transaction.state.nodes[child.0].domain_type == child_type)
        .count();
    let bytes = accounting::attachment_bytes(
        bytes,
        parent_key,
        existing,
        children.len(),
        entities,
        child_type,
    )
    .ok_or_else(|| logical_overflow(&span))?;
    transaction.execution.reserve_domain_graph(0, bytes, span)?;
    for child in &children {
        transaction.state.nodes[child.0].owner = Some(parent);
    }
    transaction.state.nodes[parent.0].children.extend(children);
    Ok(transaction.state.update(
        &transaction.scope,
        contract.id(),
        operands,
        parent,
        bytes,
        provenance,
    ))
}
