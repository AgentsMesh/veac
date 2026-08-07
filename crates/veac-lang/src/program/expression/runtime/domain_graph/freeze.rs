use std::ops::Range;
use std::sync::Arc;

use crate::program::DomainType;

use super::super::super::{DomainValue, ExpressionError, Value};
use super::state::{ArenaState, NodeId};
use super::validation;
use super::{error, DomainGraphTransaction};

mod relation;

pub(super) fn validate(
    transaction: &DomainGraphTransaction<'_>,
    root: &Value,
    span: Range<usize>,
) -> Result<(Arc<DomainValue>, NodeId), ExpressionError> {
    let node = validation::node(transaction, root, DomainType::Project, &span).map_err(|_| {
        error(
            "DOMAIN_ROOT_INVALID",
            "graph root must be this transaction's current Project handle",
            span.clone(),
        )
    })?;
    let Value::Domain(root) = root else {
        unreachable!("root was verified as a domain handle")
    };
    if transaction.state.nodes[node.0].owner.is_some() {
        return Err(error(
            "DOMAIN_ROOT_OWNED",
            "the project root cannot have an owning parent",
            span,
        ));
    }
    relation::validate(&transaction.state, &span)?;
    connected(&transaction.state, node, &span)?;
    entry(&transaction.state, node, &span)?;
    Ok((root.clone(), node))
}

pub(super) fn logical_key(state: &ArenaState, node: NodeId) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    let mut cursor = Some(node);
    for _ in 0..=state.nodes.len() {
        let value = &state.nodes[cursor?.0];
        parts.push(value.key.as_deref()?);
        cursor = value.owner;
        if cursor.is_none() {
            parts.reverse();
            return Some(parts);
        }
    }
    None
}

fn connected(state: &ArenaState, root: NodeId, span: &Range<usize>) -> Result<(), ExpressionError> {
    for (index, _) in state
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.entity)
    {
        let mut cursor = NodeId(index);
        let mut reaches_root = cursor == root;
        for _ in 0..state.nodes.len() {
            let Some(owner) = state.nodes[cursor.0].owner else {
                break;
            };
            cursor = owner;
            reaches_root = cursor == root;
            if reaches_root {
                break;
            }
        }
        if !reaches_root {
            return Err(error(
                "DOMAIN_PROJECT_DISCONNECTED",
                "every emitted graph entity must be connected to the returned Project",
                span.clone(),
            ));
        }
    }
    Ok(())
}

fn entry(state: &ArenaState, root: NodeId, span: &Range<usize>) -> Result<(), ExpressionError> {
    let project = &state.nodes[root.0];
    let entry = project.entry.ok_or_else(|| {
        error(
            "DOMAIN_PROJECT_ENTRY_REQUIRED",
            "the returned Project requires one entry sequence key",
            span.clone(),
        )
    })?;
    let entry_node = state.nodes.get(entry.0);
    let valid = entry_node.is_some_and(|node| {
        node.domain_type == DomainType::Sequence
            && node.owner == Some(root)
            && project.children.contains(&entry)
    });
    valid.then_some(()).ok_or_else(|| {
        error(
            "DOMAIN_PROJECT_ENTRY_INVALID",
            "project entry does not reference an owned Sequence",
            span.clone(),
        )
    })
}
