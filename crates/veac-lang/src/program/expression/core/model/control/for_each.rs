use std::ops::Range;

use super::super::super::{
    BlockId, ClosureDefinitionId, CoreTypeId, CoreValueMetadata, Effect, ValueId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreForEachOrder {
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreForEachSlotId {
    Element,
    Index,
}

impl CoreForEachSlotId {
    pub const fn parameter_index(self) -> usize {
        match self {
            Self::Element => 0,
            Self::Index => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreForEachSlot {
    pub(crate) id: CoreForEachSlotId,
    pub(crate) type_id: CoreTypeId,
}

impl CoreForEachSlot {
    pub const fn id(self) -> CoreForEachSlotId {
        self.id
    }

    pub const fn type_id(self) -> CoreTypeId {
        self.type_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreForEachProvenance {
    pub(crate) definition: ClosureDefinitionId,
    pub(crate) loop_span: Range<usize>,
    pub(crate) binding_span: Range<usize>,
}

impl CoreForEachProvenance {
    pub const fn definition(&self) -> ClosureDefinitionId {
        self.definition
    }

    pub const fn loop_span(&self) -> &Range<usize> {
        &self.loop_span
    }

    pub const fn binding_span(&self) -> &Range<usize> {
        &self.binding_span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreForEachEffect {
    pub(crate) summary: Effect,
    pub(crate) contains_local_mutation: bool,
}

impl CoreForEachEffect {
    pub(crate) fn from_metadata(value: &CoreValueMetadata) -> Self {
        Self {
            summary: value.effect(),
            contains_local_mutation: value.contains_local_mutation(),
        }
    }

    pub const fn summary(self) -> Effect {
        self.summary
    }

    pub const fn contains_local_mutation(self) -> bool {
        self.contains_local_mutation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreForEach {
    pub(crate) iterable: ValueId,
    pub(crate) maximum_count: u32,
    pub(crate) order: CoreForEachOrder,
    pub(crate) element: CoreForEachSlot,
    pub(crate) index: CoreForEachSlot,
    pub(crate) body: ClosureDefinitionId,
    pub(crate) captures: Vec<ValueId>,
    pub(crate) continuation: BlockId,
    pub(crate) provenance: CoreForEachProvenance,
    pub(crate) effect: CoreForEachEffect,
    pub(crate) result_metadata: CoreValueMetadata,
}

impl CoreForEach {
    pub const fn iterable(&self) -> ValueId {
        self.iterable
    }

    pub const fn maximum_count(&self) -> u32 {
        self.maximum_count
    }

    pub const fn order(&self) -> CoreForEachOrder {
        self.order
    }

    pub const fn element(&self) -> CoreForEachSlot {
        self.element
    }

    pub const fn index(&self) -> CoreForEachSlot {
        self.index
    }

    pub const fn body(&self) -> ClosureDefinitionId {
        self.body
    }

    pub fn captures(&self) -> &[ValueId] {
        &self.captures
    }

    pub const fn continuation(&self) -> BlockId {
        self.continuation
    }

    pub const fn provenance(&self) -> &CoreForEachProvenance {
        &self.provenance
    }

    pub const fn effect(&self) -> CoreForEachEffect {
        self.effect
    }

    pub fn result_metadata(&self) -> &CoreValueMetadata {
        &self.result_metadata
    }

    pub(super) fn operands(&self) -> std::vec::IntoIter<ValueId> {
        let mut values = Vec::with_capacity(self.captures.len() + 1);
        values.push(self.iterable);
        values.extend(&self.captures);
        values.into_iter()
    }
}
