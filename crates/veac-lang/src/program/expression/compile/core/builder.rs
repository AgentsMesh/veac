use super::Builder;
use crate::program::expression::core::{
    CoreInstruction, CoreInstructionKind, CoreTypeId, CoreValueMetadata, ValueId,
};
use crate::program::expression::ValueType;

mod input;
mod metadata;

impl Builder<'_> {
    pub(super) fn emit(
        &mut self,
        kind: CoreInstructionKind,
        value_type: &ValueType,
        span: std::ops::Range<usize>,
    ) -> ValueId {
        let metadata = self.instruction_metadata(&kind, value_type);
        let type_id = self.types.intern_value(value_type);
        self.emit_core(kind, type_id, metadata, span)
    }

    pub(super) fn emit_core(
        &mut self,
        kind: CoreInstructionKind,
        type_id: CoreTypeId,
        metadata: CoreValueMetadata,
        span: std::ops::Range<usize>,
    ) -> ValueId {
        let id = self.value_id();
        self.current_mut().instructions.push(CoreInstruction {
            id,
            kind,
            type_id,
            metadata: metadata.clone(),
            span,
        });
        self.metadata.insert(id, metadata);
        id
    }

    pub(super) fn value_id(&mut self) -> ValueId {
        let id = ValueId::new(self.next_value);
        self.next_value = self
            .next_value
            .checked_add(1)
            .expect("expression node limit bounds Core value IDs");
        id
    }

    pub(super) fn metadata(&self, id: ValueId) -> CoreValueMetadata {
        self.metadata
            .get(&id)
            .expect("lowered Core value has metadata")
            .clone()
    }

    pub(super) fn current_mut(&mut self) -> &mut super::PendingBlock {
        &mut self.blocks[self.current.index().expect("block ID fits usize")]
    }
}
