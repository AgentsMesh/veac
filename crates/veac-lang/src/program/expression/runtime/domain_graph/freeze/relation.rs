use std::collections::BTreeSet;
use std::ops::Range;

use crate::program::expression::ExpressionError;
use crate::program::DomainType;

use super::super::error;
use super::super::state::{ArenaState, NodeId};

pub(super) fn validate(state: &ArenaState, span: &Range<usize>) -> Result<(), ExpressionError> {
    for relation in state
        .nodes
        .iter()
        .filter(|node| node.domain_type == DomainType::Relation || node.relation.is_some())
    {
        if relation.domain_type != DomainType::Relation || !relation.children.is_empty() {
            return Err(invalid(span));
        }
        let owner = relation.owner.ok_or_else(|| invalid(span))?;
        if state.nodes.get(owner.0).map(|node| node.domain_type) != Some(DomainType::Sequence) {
            return Err(invalid(span));
        }
        let references = relation.relation.as_ref().ok_or_else(|| invalid(span))?;
        let references = references.as_slice();
        let unique = references.iter().copied().collect::<BTreeSet<_>>();
        if references.is_empty() || unique.len() != references.len() {
            return Err(invalid(span));
        }
        for reference in references {
            let node = state.nodes.get(reference.0).ok_or_else(|| invalid(span))?;
            if !node.entity
                || !node.domain_type.is_graph_entity()
                || has_relation_owner(state, *reference)
                || owning_sequence(state, *reference) != Some(owner)
            {
                return Err(invalid(span));
            }
        }
    }
    Ok(())
}

fn has_relation_owner(state: &ArenaState, node: NodeId) -> bool {
    let mut cursor = state.nodes.get(node.0).and_then(|value| value.owner);
    for _ in 0..=state.nodes.len() {
        let Some(current) = cursor else { return false };
        let Some(value) = state.nodes.get(current.0) else {
            return true;
        };
        if value.domain_type == DomainType::Relation {
            return true;
        }
        cursor = value.owner;
    }
    true
}

fn owning_sequence(state: &ArenaState, node: NodeId) -> Option<NodeId> {
    let mut cursor = Some(node);
    for _ in 0..=state.nodes.len() {
        let current = cursor?;
        let value = state.nodes.get(current.0)?;
        if value.domain_type == DomainType::Sequence {
            return Some(current);
        }
        cursor = value.owner;
    }
    None
}

fn invalid(span: &Range<usize>) -> ExpressionError {
    error(
        "DOMAIN_RELATION_TOPOLOGY",
        "a Relation must belong to one Sequence and reference unique graph entities in that Sequence",
        span.clone(),
    )
}
