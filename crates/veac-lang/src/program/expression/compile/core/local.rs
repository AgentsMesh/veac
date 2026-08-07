use super::Builder;
use crate::program::expression::core::{
    CoreInstructionKind, CoreLocalSlot, CoreValueMetadata, LocalSlotId,
};
use crate::program::expression::hir::{
    MutableLocalId, TypedMutableAssignment, TypedMutableBinding, TypedNode,
};

mod ambient;

impl Builder<'_> {
    pub(super) fn local_init(&mut self, binding: &TypedMutableBinding) {
        let value = self.node(&binding.value);
        let slot = LocalSlotId::new(
            u32::try_from(self.local_slots.len()).expect("local slot limit fits u32"),
        );
        let type_id = self.types.intern_value(&binding.value.value_type);
        self.local_slots.push(CoreLocalSlot {
            id: slot,
            type_id,
            metadata: CoreValueMetadata::local_mutation([&self.ambient]),
            span: binding.value.span.clone(),
        });
        assert!(
            self.mutable_locals.insert(binding.id, slot).is_none(),
            "typed mutable local is initialized once"
        );
        self.emit(
            CoreInstructionKind::LocalInit { slot, value },
            &binding.value.value_type,
            binding.value.span.clone(),
        );
    }

    pub(super) fn local_set(&mut self, assignment: &TypedMutableAssignment) {
        let slot = self.slot(assignment.id);
        let value = self.node(&assignment.value);
        self.emit(
            CoreInstructionKind::LocalSet { slot, value },
            &assignment.value.value_type,
            assignment.value.span.clone(),
        );
    }

    pub(super) fn local_get(&mut self, id: MutableLocalId, node: &TypedNode) -> super::ValueId {
        self.emit(
            CoreInstructionKind::LocalGet {
                slot: self.slot(id),
            },
            &node.value_type,
            node.span.clone(),
        )
    }

    fn slot(&self, id: MutableLocalId) -> LocalSlotId {
        *self
            .mutable_locals
            .get(&id)
            .expect("typed mutable local must be initialized")
    }
}
