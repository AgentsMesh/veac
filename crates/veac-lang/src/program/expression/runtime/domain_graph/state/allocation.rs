use std::sync::Arc;

use crate::program::DomainType;

use super::NodeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::program::expression::runtime::domain_graph) struct RelationReferences {
    nodes: Vec<NodeId>,
}

impl RelationReferences {
    pub(in crate::program::expression::runtime::domain_graph) fn new(nodes: Vec<NodeId>) -> Self {
        Self { nodes }
    }

    pub(in crate::program::expression::runtime::domain_graph) fn as_slice(&self) -> &[NodeId] {
        &self.nodes
    }
}

pub(in crate::program::expression::runtime::domain_graph) struct NodeAllocation {
    pub(super) domain_type: DomainType,
    pub(super) key: Option<Arc<str>>,
    pub(super) entity: bool,
    pub(super) logical_bytes: usize,
    pub(super) relation: Option<RelationReferences>,
}

impl NodeAllocation {
    pub(in crate::program::expression::runtime::domain_graph) fn description(
        domain_type: DomainType,
        logical_bytes: usize,
    ) -> Self {
        Self {
            domain_type,
            key: None,
            entity: false,
            logical_bytes,
            relation: None,
        }
    }

    pub(in crate::program::expression::runtime::domain_graph) fn entity(
        domain_type: DomainType,
        key: Arc<str>,
        logical_bytes: usize,
    ) -> Self {
        Self {
            domain_type,
            key: Some(key),
            entity: true,
            logical_bytes,
            relation: None,
        }
    }

    pub(in crate::program::expression::runtime::domain_graph) fn relation(
        domain_type: DomainType,
        key: Arc<str>,
        references: RelationReferences,
        logical_bytes: usize,
    ) -> Self {
        Self {
            domain_type,
            key: Some(key),
            entity: true,
            logical_bytes,
            relation: Some(references),
        }
    }
}
