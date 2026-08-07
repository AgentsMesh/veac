use super::Builder;
use crate::program::expression::core::{CoreInstructionKind, ValueId};
use crate::program::expression::hir::{
    TypedEnumConstruct, TypedNode, TypedStructConstruct, TypedStructProject,
};
use crate::program::FieldIndex;

impl Builder<'_> {
    pub(super) fn structure(&mut self, value: &TypedStructConstruct, node: &TypedNode) -> ValueId {
        let fields = self.ordered_fields(&value.fields);
        self.emit(
            CoreInstructionKind::StructConstruct {
                type_id: value.nominal,
                fields,
            },
            &node.value_type,
            node.span.clone(),
        )
    }

    pub(super) fn enumeration(&mut self, value: &TypedEnumConstruct, node: &TypedNode) -> ValueId {
        let fields = self.ordered_fields(&value.fields);
        self.emit(
            CoreInstructionKind::EnumConstruct {
                type_id: value.nominal,
                variant: value.variant,
                fields,
            },
            &node.value_type,
            node.span.clone(),
        )
    }

    pub(super) fn project(&mut self, value: &TypedStructProject, node: &TypedNode) -> ValueId {
        let structure = self.node(&value.receiver);
        self.emit(
            CoreInstructionKind::StructProject {
                structure,
                field: value.field,
            },
            &node.value_type,
            node.span.clone(),
        )
    }

    fn ordered_fields(&mut self, fields: &[(FieldIndex, TypedNode)]) -> Vec<ValueId> {
        let mut ordered = vec![None; fields.len()];
        for (field, value) in fields {
            let value = self.node(value);
            assert!(
                ordered[field.index()].replace(value).is_none(),
                "typed nominal fields are unique and dense"
            );
        }
        ordered
            .into_iter()
            .map(|value| value.expect("typed nominal fields cover their layout"))
            .collect()
    }
}
