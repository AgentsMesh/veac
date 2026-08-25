use super::Builder;
use crate::program::expression::core::{CoreInstructionKind, ValueId};
use crate::program::expression::hir::{TypedDomainCall, TypedNode};
use crate::program::DomainInstructionKind;

impl Builder<'_> {
    pub(super) fn domain_call(&mut self, call: &TypedDomainCall, node: &TypedNode) -> ValueId {
        let operands = self.complete_arguments(&call.operands);
        let contract = self
            .domain
            .lookup_opcode(call.opcode)
            .expect("typed domain opcode is registered");
        let kind = match contract.instruction() {
            DomainInstructionKind::DomainConstruct => CoreInstructionKind::DomainConstruct {
                opcode: call.opcode,
                operands,
            },
            DomainInstructionKind::GraphEmit => CoreInstructionKind::GraphEmit {
                opcode: call.opcode,
                operands,
            },
        };
        self.emit(kind, &node.value_type, node.span.clone())
    }

    pub(super) fn domain_metadata(
        &self,
        opcode: u16,
        operands: &[ValueId],
    ) -> crate::program::expression::CoreValueMetadata {
        let contract = self
            .domain
            .lookup_opcode(opcode)
            .expect("typed domain opcode is registered");
        crate::program::expression::CoreValueMetadata::domain(
            contract,
            operands.iter().map(|operand| {
                self.metadata
                    .get(operand)
                    .expect("lowered Core operand has metadata")
            }),
        )
    }
}
