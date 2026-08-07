use super::Builder;
use crate::program::expression::core::{
    CoreForEach, CoreForEachEffect, CoreForEachOrder, CoreForEachProvenance, CoreForEachSlot,
    CoreForEachSlotId, CoreTerminator, CoreValueMetadata,
};
use crate::program::expression::hir::{TypedIteration, TypedNode, TypedNodeKind};
use crate::program::expression::{PrimitiveType, ValueType, ValueTypeKind};

impl Builder<'_> {
    pub(super) fn for_each(
        &mut self,
        iterable: &TypedNode,
        body: &TypedNode,
        iteration: &TypedIteration,
        node: &TypedNode,
    ) -> crate::program::expression::ValueId {
        let iterable_id = self.node(iterable);
        let TypedNodeKind::Closure {
            parameters,
            captures,
            body: block,
            non_escaping,
        } = &body.kind
        else {
            unreachable!("typed for body is a synthesized closure")
        };
        assert!(*non_escaping, "typed for body cannot escape");
        let index_type = ValueType::primitive(PrimitiveType::Integer);
        let defined = self.define_closure(
            parameters,
            std::slice::from_ref(&index_type),
            captures,
            block,
            true,
            body,
        );
        let ValueTypeKind::Function { result, .. } = body.value_type.kind() else {
            unreachable!("typed for body has a function type")
        };
        let captures = defined
            .captures
            .iter()
            .map(|id| self.metadata(*id))
            .collect::<Vec<_>>();
        let result_metadata = CoreValueMetadata::for_each(
            &self.metadata(iterable_id),
            &defined.summary,
            &captures,
            result,
            &node.value_type,
        )
        .expect("typed for body has closed metadata");
        let continuation = self.new_block();
        let result_id = self.join_parameter(
            continuation,
            &node.value_type,
            node.span.clone(),
            result_metadata.clone(),
        );
        let maximum_count =
            u32::try_from(crate::program::expression::execution_budget::MAX_EXECUTION_ITERATIONS)
                .expect("iteration limit fits u32");
        let element = CoreForEachSlot {
            id: CoreForEachSlotId::Element,
            type_id: self.types.intern_value(&parameters[0].value_type),
        };
        let index = CoreForEachSlot {
            id: CoreForEachSlotId::Index,
            type_id: self.types.intern_value(&index_type),
        };
        self.terminate(CoreTerminator::ForEach(CoreForEach {
            iterable: iterable_id,
            maximum_count,
            order: CoreForEachOrder::Source,
            element,
            index,
            body: defined.id,
            captures: defined.captures,
            continuation,
            provenance: CoreForEachProvenance {
                definition: defined.id,
                loop_span: node.span.clone(),
                binding_span: iteration.binding_span.clone(),
            },
            effect: CoreForEachEffect::from_metadata(&result_metadata),
            result_metadata,
        }));
        self.current = continuation;
        result_id
    }
}
