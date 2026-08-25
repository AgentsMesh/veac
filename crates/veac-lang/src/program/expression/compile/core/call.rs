use super::Builder;
use crate::program::expression::core::{CoreCallTarget, CoreInstructionKind, ValueId};
use crate::program::expression::hir::{TypedMethodCall, TypedNode};

impl Builder<'_> {
    pub(super) fn method_call(&mut self, call: &TypedMethodCall, node: &TypedNode) -> ValueId {
        let receiver = self.node(&call.receiver);
        let mut arguments = Vec::with_capacity(call.arguments.len() + call.defaults.len() + 1);
        arguments.push(receiver);
        arguments.extend(self.call_arguments(&call.arguments, &call.defaults, node.span.clone()));
        self.emit(
            CoreInstructionKind::Call {
                target: CoreCallTarget::User(call.target),
                arguments,
            },
            &node.value_type,
            node.span.clone(),
        )
    }
}
