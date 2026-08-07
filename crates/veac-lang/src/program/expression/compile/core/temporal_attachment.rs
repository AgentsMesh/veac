use super::Builder;
use crate::program::expression::core::{CoreInstructionKind, ValueId};
use crate::program::expression::hir::{TypedNode, TypedTemporalAttach};

impl Builder<'_> {
    pub(super) fn temporal_attachment(
        &mut self,
        value: &TypedTemporalAttach,
        node: &TypedNode,
    ) -> ValueId {
        let owner = self.node(&value.owner);
        let selectors = value
            .selectors
            .iter()
            .map(|value| self.node(value))
            .collect();
        let animation = self.node(&value.animation);
        self.emit(
            CoreInstructionKind::TemporalAttach {
                kind: value.kind.opcode(),
                owner,
                selectors,
                animation,
            },
            &node.value_type,
            node.span.clone(),
        )
    }
}
