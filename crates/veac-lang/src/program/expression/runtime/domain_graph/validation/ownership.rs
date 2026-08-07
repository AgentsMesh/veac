use std::collections::BTreeSet;
use std::ops::Range;

use crate::program::expression::ExpressionError;

use super::super::state::NodeId;
use super::super::{error, DomainGraphTransaction};

pub(in crate::program::expression::runtime::domain_graph) fn attachment(
    transaction: &DomainGraphTransaction<'_>,
    parent: NodeId,
    children: &[NodeId],
    span: &Range<usize>,
) -> Result<(), ExpressionError> {
    let parent_node = &transaction.state.nodes[parent.0];
    if parent_node.owner.is_some() {
        return Err(error(
            "DOMAIN_TOPOLOGY_CLOSED",
            "an attached graph container cannot be updated",
            span.clone(),
        ));
    }
    let mut nodes = BTreeSet::new();
    let mut keys = parent_node
        .children
        .iter()
        .filter_map(|id| {
            let child = &transaction.state.nodes[id.0];
            Some((child.domain_type, child.key.clone()?))
        })
        .collect::<BTreeSet<_>>();
    for child in children {
        let value = &transaction.state.nodes[child.0];
        if !nodes.insert(*child) || value.owner.is_some() {
            return Err(error(
                "DOMAIN_SINGLE_OWNERSHIP",
                "a graph entity can have exactly one owning parent",
                span.clone(),
            ));
        }
        if transaction.state.would_cycle(parent, *child) {
            return Err(error(
                "DOMAIN_OWNERSHIP_CYCLE",
                "graph ownership cannot contain a cycle",
                span.clone(),
            ));
        }
        let key = value.key.clone().ok_or_else(|| {
            error(
                "DOMAIN_KEY_REQUIRED",
                "attached graph entities require an explicit identifier key",
                span.clone(),
            )
        })?;
        if !keys.insert((value.domain_type, key.clone())) {
            return Err(error(
                "DOMAIN_DUPLICATE_KEY",
                format!("child key `{key}` occurs more than once in its parent"),
                span.clone(),
            ));
        }
    }
    Ok(())
}

pub(in crate::program::expression::runtime::domain_graph) fn unowned(
    transaction: &DomainGraphTransaction<'_>,
    node: NodeId,
    span: &Range<usize>,
) -> Result<(), ExpressionError> {
    if transaction.state.nodes[node.0].owner.is_some() {
        return Err(error(
            "DOMAIN_TOPOLOGY_CLOSED",
            "an attached graph entity cannot be updated",
            span.clone(),
        ));
    }
    Ok(())
}
