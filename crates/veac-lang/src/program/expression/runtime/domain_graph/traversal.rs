use crate::program::expression::Value;
use crate::program::{DomainOperationId, DomainType};

use super::state::NodeId;
use super::FrozenDomainGraph;
use super::{freeze, state};
use crate::program::expression::DomainValue;

mod provenance;
mod temporal;
pub(in crate::program) use temporal::FrozenTemporalAttachment;

#[derive(Clone, Copy)]
pub(in crate::program) struct FrozenEntity<'a> {
    graph: &'a FrozenDomainGraph,
    node: NodeId,
}

impl<'a> FrozenEntity<'a> {
    pub(super) fn new(graph: &'a FrozenDomainGraph, node: NodeId) -> Self {
        Self { graph, node }
    }

    pub(in crate::program) fn domain_type(self) -> Option<DomainType> {
        Some(self.graph.state.nodes.get(self.node.0)?.domain_type)
    }

    pub(in crate::program) fn key(self) -> Option<&'a str> {
        self.graph.state.nodes.get(self.node.0)?.key.as_deref()
    }

    pub(in crate::program) fn logical_path(self) -> Option<Vec<&'a str>> {
        super::freeze::logical_key(&self.graph.state, self.node)
    }

    pub(in crate::program) fn constructor(self) -> Option<(DomainOperationId, &'a [Value])> {
        let node = self.graph.state.nodes.get(self.node.0)?;
        let record = self.graph.state.records.get(node.origin_slot as usize)?;
        let operation = record.operation?;
        (record.node == self.node).then_some((operation, record.operands.as_ref()))
    }

    pub(in crate::program) fn latest_update(
        self,
        operation: DomainOperationId,
    ) -> Option<&'a [Value]> {
        let node = self.graph.state.nodes.get(self.node.0)?;
        let start = node.origin_slot as usize;
        let end = node.latest_slot as usize;
        self.graph
            .state
            .records
            .get(start..=end)?
            .iter()
            .rev()
            .find(|record| record.node == self.node && record.operation == Some(operation))
            .map(|record| record.operands.as_ref())
    }

    pub(in crate::program) fn updates(
        self,
        operation: DomainOperationId,
    ) -> impl Iterator<Item = &'a [Value]> {
        let range = self.graph.state.nodes.get(self.node.0).and_then(|node| {
            self.graph
                .state
                .records
                .get(node.origin_slot as usize..=node.latest_slot as usize)
        });
        range
            .into_iter()
            .flatten()
            .filter(move |record| record.node == self.node && record.operation == Some(operation))
            .map(|record| record.operands.as_ref())
    }

    pub(in crate::program) fn children(self) -> impl Iterator<Item = Self> + 'a {
        self.graph
            .state
            .nodes
            .get(self.node.0)
            .into_iter()
            .flat_map(|node| node.children.iter().copied())
            .map(move |node| Self::new(self.graph, node))
    }

    pub(in crate::program) fn relation_endpoints(self) -> Option<(Self, Self)> {
        let references = self.graph.state.nodes.get(self.node.0)?.relation.as_ref()?;
        let from = *references.as_slice().first()?;
        let to = *references.as_slice().get(1)?;
        Some((Self::new(self.graph, from), Self::new(self.graph, to)))
    }

    pub(in crate::program) fn relation_references(self) -> impl Iterator<Item = Self> + 'a {
        self.graph
            .state
            .nodes
            .get(self.node.0)
            .and_then(|node| node.relation.as_ref())
            .into_iter()
            .flat_map(|references| references.as_slice().iter().copied())
            .map(move |node| Self::new(self.graph, node))
    }
}

impl FrozenDomainGraph {
    pub(in crate::program) fn root_entity(&self) -> FrozenEntity<'_> {
        FrozenEntity::new(self, self.root_node)
    }

    pub(in crate::program) fn entry_key(&self) -> Option<&str> {
        let entry = self.state.nodes.get(self.root_node.0)?.entry?;
        self.state.nodes.get(entry.0)?.key.as_deref()
    }

    pub(in crate::program) fn root(&self) -> &DomainValue {
        &self.root
    }

    pub(in crate::program) fn record_count(&self) -> usize {
        self.state.records.len()
    }

    pub(in crate::program) fn entity_count(&self) -> usize {
        self.state.nodes.iter().filter(|node| node.entity).count()
    }

    pub(in crate::program) const fn logical_bytes(&self) -> usize {
        self.state.logical_bytes
    }

    pub(in crate::program) fn operation(&self, value: &DomainValue) -> Option<DomainOperationId> {
        self.record(value)?.operation
    }

    pub(in crate::program) fn operands(&self, value: &DomainValue) -> Option<&[Value]> {
        Some(&self.record(value)?.operands)
    }

    pub(in crate::program) fn logical_key(&self, value: &DomainValue) -> Option<Vec<&str>> {
        freeze::logical_key(&self.state, self.record(value)?.node)
    }

    pub(in crate::program) fn typed_program_identity(
        &self,
    ) -> Option<&crate::program::expression::ProgramIdentity> {
        self.identity.as_ref()
    }

    fn record(&self, value: &DomainValue) -> Option<&state::DomainRecord> {
        value
            .belongs_to(&self.scope)
            .then(|| self.state.records.get(value.arena_slot() as usize))
            .flatten()
    }

    pub(in crate::program) fn root_logical_key(&self) -> Vec<&str> {
        freeze::logical_key(&self.state, self.root_node).expect("verified project key")
    }
}
