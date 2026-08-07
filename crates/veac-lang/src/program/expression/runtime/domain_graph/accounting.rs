use super::super::super::Value;
use crate::program::{DomainType, DomainValueShape};

use super::provenance;

pub(in crate::program::expression) const LOGICAL_DOMAIN_RECORD_BYTES: usize = 128;
pub(in crate::program::expression) const LOGICAL_DOMAIN_OPERAND_BYTES: usize = 64;
pub(in crate::program::expression) const LOGICAL_GRAPH_REFERENCE_BYTES: usize = 16;
const LOGICAL_TEMPORAL_ATTACHMENT_BYTES: usize = 128;
const TRACK_PROVENANCE_EMPTY_OBJECT_BYTES: usize = 2;
const CANONICAL_TRACK_ID_JSON_BYTES: usize = 2 + 4 + 64;
const TRACK_PROVENANCE_ENTRY_BYTES: usize = CANONICAL_TRACK_ID_JSON_BYTES + 1;
const RELATION_PROVENANCE_EMPTY_OBJECT_BYTES: usize = 2;
const CANONICAL_RELATION_ID_JSON_BYTES: usize = 2 + 4 + 64;
const RELATION_PROVENANCE_ENTRY_BYTES: usize = CANONICAL_RELATION_ID_JSON_BYTES + 1;

pub(super) fn record_bytes(
    result: DomainValueShape,
    operands: &[Value],
    provenance_bytes: usize,
) -> Option<usize> {
    let bytes = totals(
        LOGICAL_DOMAIN_RECORD_BYTES.checked_add(provenance_bytes)?,
        operands.len(),
        operands.iter().map(Value::retained_bytes),
    )?;
    bytes.checked_add(match result.domain_type() {
        Some(DomainType::Sequence) => {
            TRACK_PROVENANCE_EMPTY_OBJECT_BYTES + RELATION_PROVENANCE_EMPTY_OBJECT_BYTES
        }
        _ => 0,
    })
}

pub(super) fn attachment_bytes(
    bytes: usize,
    parent_key: &str,
    existing_children: usize,
    added_children: usize,
    provenanced_entities: usize,
    child_type: DomainType,
) -> Option<usize> {
    let path = provenance::path_extension_bytes(parent_key, provenanced_entities)?;
    let tracks = match child_type {
        DomainType::Layer => track_index_delta(existing_children, added_children)?,
        _ => 0,
    };
    let relations = match child_type {
        DomainType::Relation => index_delta(
            existing_children,
            added_children,
            RELATION_PROVENANCE_ENTRY_BYTES,
        )?,
        _ => 0,
    };
    bytes
        .checked_add(path)?
        .checked_add(tracks)?
        .checked_add(relations)
}

pub(super) fn reference_bytes(count: usize) -> Option<usize> {
    count.checked_mul(LOGICAL_GRAPH_REFERENCE_BYTES)
}

pub(super) fn temporal_attachment_bytes(
    owner: &Value,
    selectors: &[Value],
    animation: &Value,
) -> Option<usize> {
    totals(
        LOGICAL_TEMPORAL_ATTACHMENT_BYTES,
        selectors.len().checked_add(2)?,
        std::iter::once(owner)
            .chain(selectors)
            .chain(std::iter::once(animation))
            .map(Value::retained_bytes),
    )
}

fn track_index_delta(existing: usize, added: usize) -> Option<usize> {
    index_delta(existing, added, TRACK_PROVENANCE_ENTRY_BYTES)
}

fn index_delta(existing: usize, added: usize, entry_bytes: usize) -> Option<usize> {
    let entries = entry_bytes.checked_mul(added)?;
    let separators = if existing == 0 {
        added.saturating_sub(1)
    } else {
        added
    };
    entries.checked_add(separators)
}

pub(super) fn totals(
    base: usize,
    operand_count: usize,
    retained: impl IntoIterator<Item = usize>,
) -> Option<usize> {
    let slots = operand_count.checked_mul(LOGICAL_DOMAIN_OPERAND_BYTES)?;
    retained
        .into_iter()
        .try_fold(base.checked_add(slots)?, usize::checked_add)
}
