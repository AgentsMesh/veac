use std::ops::Range;

use super::super::{CoreTypeId, CoreValueMetadata, LocalSlotId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreLocalSlot {
    pub(crate) id: LocalSlotId,
    pub(crate) type_id: CoreTypeId,
    pub(crate) metadata: CoreValueMetadata,
    pub(crate) span: Range<usize>,
}

impl CoreLocalSlot {
    pub fn id(&self) -> LocalSlotId {
        self.id
    }

    pub fn type_id(&self) -> CoreTypeId {
        self.type_id
    }

    pub fn metadata(&self) -> &CoreValueMetadata {
        &self.metadata
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}
