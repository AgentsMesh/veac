use super::Builder;
use crate::program::expression::core::CoreInstructionKind;
use crate::program::expression::hir::TypedNode;
use crate::program::expression::{
    CollectionOperation, PrimitiveType, ValueId, ValueType, ValueTypeKind,
};

impl Builder<'_> {
    pub(super) fn aggregate(
        &mut self,
        operation: CollectionOperation,
        arguments: &[TypedNode],
        node: &TypedNode,
    ) -> ValueId {
        let values = arguments
            .iter()
            .map(|argument| self.node(argument))
            .collect::<Vec<_>>();
        let (iterable, initial, callable) = match operation {
            CollectionOperation::Map | CollectionOperation::Filter => (values[0], None, values[1]),
            CollectionOperation::Fold => (values[0], Some(values[1]), values[2]),
        };
        self.emit(
            CoreInstructionKind::Collection {
                operation,
                iterable,
                initial,
                callable,
            },
            &node.value_type,
            node.span.clone(),
        )
    }
}

pub(super) fn callback_result(operation: CollectionOperation, result: &ValueType) -> ValueType {
    match operation {
        CollectionOperation::Map => {
            let ValueTypeKind::List(element) = result.kind() else {
                unreachable!("typed map result is a list")
            };
            element.clone()
        }
        CollectionOperation::Filter => ValueType::primitive(PrimitiveType::Boolean),
        CollectionOperation::Fold => result.clone(),
    }
}
