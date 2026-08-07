use super::{CoreValueMetadata, EffectEvidence, Stage};
use crate::program::{DomainOperationContract, OperandAxis};

impl CoreValueMetadata {
    pub(crate) fn domain<'a>(
        contract: &DomainOperationContract,
        operands: impl ExactSizeIterator<Item = &'a Self>,
    ) -> Self {
        assert_eq!(operands.len(), contract.operands().len());
        let mut output = Self::constant();
        for (operand, specification) in operands.zip(contract.operands()) {
            match specification.axis() {
                OperandAxis::Topology => output.absorb_shape(operand),
                OperandAxis::Leaf => output.absorb_leaf(operand),
            }
        }
        output.shape_stage = output.shape_stage.max(Stage::Build);
        output.effect = output
            .effect
            .join(EffectEvidence::from_effect(contract.effect()));
        output
    }
}
