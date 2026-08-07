use super::Builder;
use crate::program::expression::core::{CoreInstructionKind, ValueId};
use crate::program::expression::hir::TypedNode;

impl Builder<'_> {
    pub(super) fn range(
        &mut self,
        start: &TypedNode,
        end: &TypedNode,
        step: Option<&TypedNode>,
        node: &TypedNode,
    ) -> ValueId {
        let start = self.node(start);
        let end = self.node(end);
        let step = step.map(|step| self.node(step));
        self.emit(
            CoreInstructionKind::Range { start, end, step },
            &node.value_type,
            node.span.clone(),
        )
    }
}
