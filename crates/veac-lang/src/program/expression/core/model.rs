use std::ops::Range;

use super::{BlockId, CoreTypeId, CoreTypeTable, CoreValueMetadata, ValueId};
use crate::program::expression::ValueType;
use crate::program::{DomainOpsetVersion, DomainRegistryDigest};

mod closure;
mod control;
mod input;
mod instruction;
mod local;
mod nominal;
#[cfg(test)]
mod tests;
pub use closure::CoreClosureDefinition;
pub use control::{
    CoreForEach, CoreForEachEffect, CoreForEachOrder, CoreForEachProvenance, CoreForEachSlot,
    CoreForEachSlotId, CoreMatchArm, CoreTerminator,
};
pub use input::{CoreBuildInputId, CoreInputIdentity, CoreTemporalInputIdentity};
pub use input::{CoreCallableInput, CoreInput};
pub use instruction::{
    ArithmeticOperator, ComparisonOperator, CoreCallTarget, CoreInstruction, CoreInstructionKind,
    CoreTemporalComposeOperation, CoreTemporalProjectOperation, CoreUnaryOperator,
    EqualityOperator,
};
pub use local::CoreLocalSlot;
pub use nominal::CoreNominalDefinition;

pub const CORE_VERSION: u16 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreBlockParameter {
    pub(crate) id: ValueId,
    pub(crate) type_id: CoreTypeId,
    pub(crate) metadata: CoreValueMetadata,
    pub(crate) span: Range<usize>,
}

impl CoreBlockParameter {
    pub fn id(&self) -> ValueId {
        self.id
    }

    pub fn type_id(&self) -> CoreTypeId {
        self.type_id
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn metadata(&self) -> &CoreValueMetadata {
        &self.metadata
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreBlock {
    pub(crate) id: BlockId,
    pub(crate) parameters: Vec<CoreBlockParameter>,
    pub(crate) instructions: Vec<CoreInstruction>,
    pub(crate) terminator: CoreTerminator,
}

impl CoreBlock {
    pub fn id(&self) -> BlockId {
        self.id
    }

    pub fn instructions(&self) -> &[CoreInstruction] {
        &self.instructions
    }

    pub fn parameters(&self) -> &[CoreBlockParameter] {
        &self.parameters
    }

    pub fn terminator(&self) -> &CoreTerminator {
        &self.terminator
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreProgram {
    pub(crate) version: u16,
    pub(crate) domain_opset: DomainOpsetVersion,
    pub(crate) domain_registry_digest: DomainRegistryDigest,
    pub(crate) entry: BlockId,
    pub(crate) types: CoreTypeTable,
    pub(crate) nominal_definitions: Vec<CoreNominalDefinition>,
    pub(crate) inputs: Vec<CoreInput>,
    pub(crate) local_slots: Vec<CoreLocalSlot>,
    pub(crate) closure_definitions: Vec<CoreClosureDefinition>,
    pub(crate) blocks: Vec<CoreBlock>,
    pub(crate) result_type: CoreTypeId,
}

impl CoreProgram {
    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn input_declarations_digest(&self) -> super::CoreInputDeclarationDigest {
        super::input_declarations_digest(self)
    }

    pub fn domain_opset(&self) -> DomainOpsetVersion {
        self.domain_opset
    }

    pub fn domain_registry_digest(&self) -> DomainRegistryDigest {
        self.domain_registry_digest
    }

    pub fn entry(&self) -> BlockId {
        self.entry
    }

    pub fn types(&self) -> &CoreTypeTable {
        &self.types
    }

    pub fn blocks(&self) -> &[CoreBlock] {
        &self.blocks
    }

    pub fn nominal_definitions(&self) -> &[CoreNominalDefinition] {
        &self.nominal_definitions
    }

    pub fn inputs(&self) -> &[CoreInput] {
        &self.inputs
    }

    pub fn local_slots(&self) -> &[CoreLocalSlot] {
        &self.local_slots
    }

    pub fn closure_definitions(&self) -> &[CoreClosureDefinition] {
        &self.closure_definitions
    }

    pub fn closure_definition_count(&self) -> usize {
        self.closure_definitions.len()
    }

    pub fn result_type_id(&self) -> CoreTypeId {
        self.result_type
    }

    pub fn result_type(&self) -> &ValueType {
        self.types
            .value(self.result_type)
            .expect("verified Core result type is a value type")
    }

    pub fn value_type(&self, id: CoreTypeId) -> Option<&ValueType> {
        self.types.value(id)
    }

    pub(crate) fn value_count(&self) -> usize {
        self.blocks
            .iter()
            .map(|block| block.parameters.len() + block.instructions.len())
            .sum()
    }

    pub(crate) fn value_span(&self, id: ValueId) -> Option<Range<usize>> {
        self.blocks.iter().find_map(|block| {
            block
                .parameters
                .iter()
                .find(|value| value.id == id)
                .map(|value| value.span.clone())
                .or_else(|| {
                    block
                        .instructions
                        .iter()
                        .find(|value| value.id == id)
                        .map(|value| value.span.clone())
                })
        })
    }
}
