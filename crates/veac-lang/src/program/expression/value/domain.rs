use std::fmt;
use std::sync::Arc;

use crate::program::DomainType;

pub(crate) const LOGICAL_DOMAIN_HANDLE_BYTES: usize = 64;

#[derive(Clone)]
pub(in crate::program::expression) struct DomainGraphScope(Arc<GraphIdentity>);

#[derive(Debug)]
pub(super) struct GraphIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(in crate::program::expression) struct DomainValueSlot(u32);

#[derive(Clone)]
pub struct DomainValue {
    pub(super) graph: DomainGraphScope,
    pub(super) slot: DomainValueSlot,
    pub(super) domain_type: DomainType,
}

impl DomainValue {
    pub(in crate::program::expression) fn from_arena(
        graph: DomainGraphScope,
        slot: u32,
        domain_type: DomainType,
    ) -> Self {
        Self {
            graph,
            slot: DomainValueSlot(slot),
            domain_type,
        }
    }

    pub const fn domain_type(&self) -> DomainType {
        self.domain_type
    }

    pub const fn is_container(&self) -> bool {
        self.domain_type.is_container()
    }

    pub(super) const fn graph(&self) -> &DomainGraphScope {
        &self.graph
    }

    pub(in crate::program::expression) fn belongs_to(&self, graph: &DomainGraphScope) -> bool {
        &self.graph == graph
    }

    pub(in crate::program::expression) const fn arena_slot(&self) -> u32 {
        self.slot.0
    }

    pub(crate) const fn retained_bytes(&self) -> usize {
        LOGICAL_DOMAIN_HANDLE_BYTES
    }
}

impl DomainGraphScope {
    pub(in crate::program::expression) fn fresh() -> Self {
        Self(Arc::new(GraphIdentity))
    }
}

impl PartialEq for DomainGraphScope {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DomainGraphScope {}

impl PartialEq for DomainValue {
    fn eq(&self, other: &Self) -> bool {
        self.graph == other.graph
            && self.slot == other.slot
            && self.domain_type == other.domain_type
    }
}

impl Eq for DomainValue {}

impl fmt::Debug for DomainGraphScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DomainGraphScope(<opaque>)")
    }
}

impl fmt::Debug for DomainValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DomainValue")
            .field("domain_type", &self.domain_type)
            .field("handle", &"<opaque>")
            .finish()
    }
}
