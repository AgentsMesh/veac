use std::collections::BTreeMap;

use super::Builder;
use crate::program::expression::core::{
    BlockId, CoreBlockParameter, CoreMatchArm, CoreTerminator, CoreValueMetadata, ValueId,
};
use crate::program::expression::hir::{TypedMatchArm, TypedNode};
use crate::program::expression::ValueTypeKind;
use crate::program::{FieldDefinition, TypeDefinitionKind};

impl Builder<'_> {
    pub(super) fn match_value(
        &mut self,
        scrutinee: &TypedNode,
        arms: &[TypedMatchArm],
        node: &TypedNode,
    ) -> ValueId {
        let value = self.node(scrutinee);
        let targets = arms.iter().map(|_| self.new_block()).collect::<Vec<_>>();
        let join = self.new_block();
        self.terminate(CoreTerminator::Match {
            scrutinee: value,
            arms: arms
                .iter()
                .zip(&targets)
                .map(|(arm, target)| CoreMatchArm {
                    variant: arm.variant,
                    target: *target,
                })
                .collect(),
            span: node.span.clone(),
        });
        let layout = self.variant_layout(scrutinee, arms);
        let mut results = Vec::with_capacity(arms.len());
        for ((arm, target), fields) in arms.iter().zip(targets).zip(layout) {
            self.current = target;
            let parameters = self.payload_parameters(target, arm.variant, &fields, value, node);
            for binding in &arm.bindings {
                self.locals.insert(binding.id, parameters[&binding.field]);
            }
            let result = self.typed_block(&arm.body);
            self.jump(join, result, arm.body.result.span.clone());
            results.push(result);
        }
        let result_metadata = results
            .iter()
            .map(|value| self.metadata(*value))
            .collect::<Vec<_>>();
        let mut metadata = CoreValueMetadata::combine(result_metadata.iter());
        if arms.len() > 1 {
            let scrutinee = self.metadata(value);
            metadata = CoreValueMetadata::combine([&metadata, &scrutinee]);
        }
        if matches!(
            node.value_type.kind(),
            ValueTypeKind::Nominal(_) | ValueTypeKind::Function { .. }
        ) {
            metadata.join_contract_from(&result_metadata.iter().collect::<Vec<_>>());
        }
        let result = self.join_parameter(join, &node.value_type, node.span.clone(), metadata);
        self.current = join;
        result
    }

    fn variant_layout(
        &self,
        scrutinee: &TypedNode,
        arms: &[TypedMatchArm],
    ) -> Vec<Vec<FieldDefinition>> {
        let ValueTypeKind::Nominal(reference) = scrutinee.value_type.kind() else {
            unreachable!("typed match scrutinee is nominal")
        };
        let definition = self
            .nominal_types
            .definition(reference.id())
            .expect("typed match nominal belongs to its registry");
        let TypeDefinitionKind::Enum(layout) = definition.kind() else {
            unreachable!("typed match scrutinee is an enum")
        };
        arms.iter()
            .map(|arm| layout.variants()[arm.variant.index()].fields().to_vec())
            .collect()
    }

    fn payload_parameters(
        &mut self,
        block: BlockId,
        variant: crate::program::VariantIndex,
        fields: &[FieldDefinition],
        scrutinee: ValueId,
        node: &TypedNode,
    ) -> BTreeMap<crate::program::FieldIndex, ValueId> {
        fields
            .iter()
            .map(|field| {
                let metadata = self
                    .metadata(scrutinee)
                    .enum_field(variant, field.index(), field.value_type())
                    .expect("typed enum payload has a closed metadata contract");
                let id = self.value_id();
                let type_id = self.types.intern_value(field.value_type());
                self.blocks[block.index().expect("block ID fits usize")]
                    .parameters
                    .push(CoreBlockParameter {
                        id,
                        type_id,
                        metadata: metadata.clone(),
                        span: node.span.clone(),
                    });
                self.metadata.insert(id, metadata);
                (field.index(), id)
            })
            .collect()
    }
}
