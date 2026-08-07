use std::sync::Arc;

use crate::program::{DomainOperationId, DomainType};

use super::super::super::value::domain::DomainGraphScope;
use super::super::super::{DomainValue, Value};
use super::provenance::DomainProvenance;
use crate::program::expression::{ClosureValue, DomainOrigin, TemporalAttachmentKind};

mod allocation;
pub(super) use allocation::{NodeAllocation, RelationReferences};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct NodeId(pub(super) usize);

#[derive(Clone)]
pub(super) struct DomainRecord {
    pub(super) operation: Option<DomainOperationId>,
    pub(super) operands: Arc<[Value]>,
    pub(super) node: NodeId,
    pub(super) provenance: Option<DomainProvenance>,
}

#[derive(Clone)]
pub(super) struct DomainNode {
    pub(super) domain_type: DomainType,
    pub(super) key: Option<Arc<str>>,
    pub(super) owner: Option<NodeId>,
    pub(super) children: Vec<NodeId>,
    pub(super) relation: Option<RelationReferences>,
    pub(super) origin_slot: u32,
    pub(super) latest_slot: u32,
    pub(super) entry: Option<NodeId>,
    pub(super) entity: bool,
}

#[derive(Clone)]
pub(super) struct TemporalAttachment {
    pub(super) owner: NodeId,
    pub(super) kind: TemporalAttachmentKind,
    pub(super) selectors: Arc<[Value]>,
    pub(super) animation: Arc<ClosureValue>,
    pub(super) instance: Option<DomainOrigin>,
}

#[derive(Clone, Default)]
pub(super) struct ArenaState {
    pub(super) records: Vec<DomainRecord>,
    pub(super) nodes: Vec<DomainNode>,
    pub(super) temporal: Vec<TemporalAttachment>,
    pub(super) logical_bytes: usize,
}

impl ArenaState {
    pub(super) fn insert(
        &mut self,
        scope: &DomainGraphScope,
        operation: DomainOperationId,
        operands: Vec<Value>,
        allocation: NodeAllocation,
        provenance: Option<DomainProvenance>,
    ) -> Value {
        self.insert_record(scope, Some(operation), operands, allocation, provenance)
    }

    pub(super) fn insert_host_context(
        &mut self,
        scope: &DomainGraphScope,
        logical_bytes: usize,
    ) -> Value {
        self.insert_record(
            scope,
            None,
            Vec::new(),
            NodeAllocation::description(DomainType::Context, logical_bytes),
            None,
        )
    }

    fn insert_record(
        &mut self,
        scope: &DomainGraphScope,
        operation: Option<DomainOperationId>,
        operands: Vec<Value>,
        allocation: NodeAllocation,
        provenance: Option<DomainProvenance>,
    ) -> Value {
        let node = NodeId(self.nodes.len());
        let slot = self.next_slot();
        self.nodes.push(DomainNode {
            domain_type: allocation.domain_type,
            key: allocation.key,
            owner: None,
            children: Vec::new(),
            relation: allocation.relation,
            origin_slot: slot,
            latest_slot: slot,
            entry: None,
            entity: allocation.entity,
        });
        self.push_record(
            operation,
            operands,
            node,
            allocation.logical_bytes,
            provenance,
        );
        Value::Domain(Arc::new(DomainValue::from_arena(
            scope.clone(),
            slot,
            allocation.domain_type,
        )))
    }

    pub(super) fn update(
        &mut self,
        scope: &DomainGraphScope,
        operation: DomainOperationId,
        operands: Vec<Value>,
        node: NodeId,
        logical_bytes: usize,
        provenance: Option<DomainProvenance>,
    ) -> Value {
        let slot = self.next_slot();
        self.nodes[node.0].latest_slot = slot;
        let domain_type = self.nodes[node.0].domain_type;
        self.push_record(Some(operation), operands, node, logical_bytes, provenance);
        Value::Domain(Arc::new(DomainValue::from_arena(
            scope.clone(),
            slot,
            domain_type,
        )))
    }

    pub(super) fn would_cycle(&self, parent: NodeId, child: NodeId) -> bool {
        let mut cursor = Some(parent);
        for _ in 0..=self.nodes.len() {
            match cursor {
                Some(value) if value == child => return true,
                Some(value) => cursor = self.nodes[value.0].owner,
                None => return false,
            }
        }
        true
    }

    pub(super) fn provenanced_entity_subtree_count(&self, roots: &[NodeId]) -> Option<usize> {
        let mut pending = roots.to_vec();
        let mut count = 0usize;
        while let Some(node) = pending.pop() {
            let value = self.nodes.get(node.0)?;
            let has_provenance = self
                .records
                .get(value.origin_slot as usize)?
                .provenance
                .is_some();
            count = count.checked_add(usize::from(value.entity && has_provenance))?;
            pending.extend(value.children.iter().copied());
        }
        Some(count)
    }

    fn next_slot(&self) -> u32 {
        u32::try_from(self.records.len()).expect("graph resource limits keep slots within u32")
    }

    fn push_record(
        &mut self,
        operation: Option<DomainOperationId>,
        operands: Vec<Value>,
        node: NodeId,
        logical_bytes: usize,
        provenance: Option<DomainProvenance>,
    ) {
        self.records.push(DomainRecord {
            operation,
            operands: operands.into(),
            node,
            provenance,
        });
        self.logical_bytes += logical_bytes;
    }
}
