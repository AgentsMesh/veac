use super::Builder;
use crate::program::expression::core::{
    CoreCallTarget, CoreInstructionKind, CoreValueMetadata, Stage,
};
use crate::program::expression::ValueType;

impl Builder<'_> {
    pub(in crate::program::expression::compile::core) fn instruction_metadata(
        &self,
        kind: &CoreInstructionKind,
        value_type: &ValueType,
    ) -> CoreValueMetadata {
        match kind {
            CoreInstructionKind::Literal(_) => CoreValueMetadata::constant(),
            CoreInstructionKind::Input(id) => {
                self.inputs[id.index().expect("dense input ID")].metadata()
            }
            CoreInstructionKind::Parameter(index) => CoreValueMetadata::typed_parameter(
                self.parameter_stages[*index],
                *index,
                value_type,
            ),
            CoreInstructionKind::Capture(index) => {
                CoreValueMetadata::capture(Stage::Const, *index, value_type)
            }
            CoreInstructionKind::LocalInit { slot, value }
            | CoreInstructionKind::LocalSet { slot, value } => {
                let slot = &self.local_slots[slot.index().expect("dense local slot ID")];
                CoreValueMetadata::local_mutation([&slot.metadata, &self.metadata(*value)])
            }
            CoreInstructionKind::LocalGet { slot } => self.local_slots
                [slot.index().expect("dense local slot ID")]
            .metadata
            .clone(),
            CoreInstructionKind::Closure {
                definition,
                captures,
            } => {
                let captures = captures
                    .iter()
                    .map(|id| self.metadata(*id))
                    .collect::<Vec<_>>();
                CoreValueMetadata::closure(
                    self.closure_definitions[definition.index().expect("dense closure ID")]
                        .summary(),
                    &captures,
                )
            }
            CoreInstructionKind::Invoke { callee, arguments } => {
                let callee = self.metadata(*callee);
                let arguments = arguments
                    .iter()
                    .map(|id| self.metadata(*id))
                    .collect::<Vec<_>>();
                CoreValueMetadata::invoke(&callee, &arguments, value_type)
                    .expect("typed Invoke callee carries a callable contract")
            }
            CoreInstructionKind::Call {
                target: CoreCallTarget::User(id),
                arguments,
            } => self.user_call_metadata(*id, arguments),
            CoreInstructionKind::Collection {
                operation,
                iterable,
                initial,
                callable,
            } => self.aggregate_metadata(*operation, *iterable, *initial, *callable, value_type),
            CoreInstructionKind::StructConstruct { fields, .. } => {
                CoreValueMetadata::structure(&self.metadata_values(fields))
            }
            CoreInstructionKind::EnumConstruct {
                variant, fields, ..
            } => CoreValueMetadata::enumeration(*variant, &self.metadata_values(fields)),
            CoreInstructionKind::List { elements } => {
                CoreValueMetadata::list(&self.metadata_values(elements))
            }
            CoreInstructionKind::Tuple { elements } => {
                CoreValueMetadata::tuple(&self.metadata_values(elements))
            }
            CoreInstructionKind::StructProject { structure, field } => self
                .metadata(*structure)
                .struct_field(*field, value_type)
                .expect("typed field projection has a closed metadata contract"),
            CoreInstructionKind::Range { start, end, step } => CoreValueMetadata::range(
                &self.metadata(*start),
                &self.metadata(*end),
                step.map(|id| self.metadata(id)).as_ref(),
            ),
            CoreInstructionKind::MapBegin { .. } => CoreValueMetadata::constant(),
            CoreInstructionKind::MapKey { builder, key, .. } => {
                let mut metadata = self.metadata(*builder);
                metadata.absorb_shape(&self.metadata(*key));
                metadata
            }
            CoreInstructionKind::MapValue { pending, value } => {
                let mut metadata = self.metadata(*pending);
                metadata.append_map_value(&self.metadata(*value));
                metadata
            }
            CoreInstructionKind::MapFinish { builder } => self.metadata(*builder),
            CoreInstructionKind::TemporalCompose { operands, .. } => {
                let values = self.metadata_values(operands);
                CoreValueMetadata::temporal_compose(values.iter())
            }
            CoreInstructionKind::TemporalProject { value, .. } => {
                CoreValueMetadata::temporal_project(&self.metadata(*value))
            }
            CoreInstructionKind::TemporalAttach {
                owner,
                selectors,
                animation,
                ..
            } => CoreValueMetadata::temporal_attachment(
                &self.metadata(*owner),
                selectors.iter().map(|value| self.metadata(*value)),
                &self.metadata(*animation),
            ),
            CoreInstructionKind::DomainConstruct { opcode, operands }
            | CoreInstructionKind::GraphEmit { opcode, operands } => {
                self.domain_metadata(*opcode, operands)
            }
            _ => {
                crate::program::expression::core::operand_metadata(kind.operands(), &self.metadata)
            }
        }
    }

    fn metadata_values(
        &self,
        values: &[crate::program::expression::ValueId],
    ) -> Vec<CoreValueMetadata> {
        values.iter().map(|id| self.metadata(*id)).collect()
    }

    fn user_call_metadata(
        &self,
        id: crate::program::expression::FunctionId,
        arguments: &[crate::program::expression::ValueId],
    ) -> CoreValueMetadata {
        self.registry
            .get(id)
            .expect("typed user call target is registered")
            .summary()
            .instantiate(&self.metadata_values(arguments), &[])
            .expect("typed call resolves every metadata binding")
    }

    fn aggregate_metadata(
        &self,
        operation: crate::program::expression::CollectionOperation,
        iterable: crate::program::expression::ValueId,
        initial: Option<crate::program::expression::ValueId>,
        callable: crate::program::expression::ValueId,
        value_type: &ValueType,
    ) -> CoreValueMetadata {
        let initial = initial.map(|id| self.metadata(id));
        let callback_result = super::super::aggregate::callback_result(operation, value_type);
        CoreValueMetadata::aggregate(
            operation,
            &self.metadata(iterable),
            initial.as_ref(),
            &self.metadata(callable),
            &callback_result,
            value_type,
        )
        .expect("typed aggregate callback carries a callable contract")
    }
}
