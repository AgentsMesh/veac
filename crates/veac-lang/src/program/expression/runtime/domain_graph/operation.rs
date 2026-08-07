use crate::program::DomainRuntimeAction;
use std::ops::Range;

use super::super::super::{DomainOrigin, ExpressionError, Value};
use super::accounting;
use super::provenance::{DomainProvenance, ProvenanceKind};
use super::validation;
use super::{error, DomainGraphTransaction};

mod actions;
mod support;
use support::kind;

pub(super) fn evaluate(
    transaction: &mut DomainGraphTransaction<'_>,
    opcode: u16,
    operands: Vec<Value>,
    span: Range<usize>,
    origin: Option<DomainOrigin>,
) -> Result<Value, ExpressionError> {
    let contract = transaction
        .registry
        .lookup_opcode(opcode)
        .cloned()
        .ok_or_else(|| {
            error(
                "DOMAIN_OPERATION_UNKNOWN",
                format!("domain opcode 0x{opcode:04x} is not in the verified operation set"),
                span.clone(),
            )
        })?;
    validation::operands(transaction, &contract, &operands, &span)?;
    validation::refinement(&contract, &operands, &span)?;
    let kind = kind(contract.runtime_action());
    let entity_key = matches!(kind, ProvenanceKind::Constructor)
        .then(|| operands.first())
        .flatten()
        .and_then(|value| match value {
            Value::Identifier(value) => Some(value.as_ref()),
            _ => None,
        });
    let provenance = origin.map(|value| DomainProvenance::new(value, kind));
    let provenance_bytes = provenance
        .as_ref()
        .map_or(0, |value| value.logical_bytes(contract.id(), entity_key));
    let bytes = accounting::record_bytes(contract.result(), &operands, provenance_bytes)
        .ok_or_else(|| {
            error(
                "DOMAIN_LOGICAL_BYTES_OVERFLOW",
                "domain graph logical storage overflows",
                span.clone(),
            )
        })?;
    match contract.runtime_action() {
        DomainRuntimeAction::Description => {
            actions::construct(transaction, &contract, operands, bytes, provenance, span)
        }
        DomainRuntimeAction::EntityConstructor => {
            actions::entity(transaction, &contract, operands, bytes, provenance, span)
        }
        DomainRuntimeAction::OwnedAttachment => {
            actions::attach(transaction, &contract, operands, bytes, provenance, span)
        }
        DomainRuntimeAction::ProjectEntry => {
            actions::entry(transaction, &contract, operands, bytes, provenance, span)
        }
        DomainRuntimeAction::NonOwningUpdate => {
            actions::update(transaction, &contract, operands, bytes, provenance, span)
        }
        DomainRuntimeAction::RelationConstructor => {
            actions::relation(transaction, &contract, operands, bytes, provenance, span)
        }
    }
}
