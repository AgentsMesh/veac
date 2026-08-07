use std::collections::BTreeMap;

use super::super::super::core::{
    BlockId, CoreBlockParameter, CoreCallableInput, CoreClosureDefinition, CoreInput,
    CoreInputIdentity, CoreInstruction, CoreLocalSlot, CoreTerminator, CoreValueMetadata,
    FunctionRegistry, InputId, LocalSlotId, ValueId,
};
use super::super::super::hir::{LocalId, MutableLocalId};
use super::type_table::TypeTableBuilder;

pub(super) struct PendingBlock {
    pub(super) id: BlockId,
    pub(super) parameters: Vec<CoreBlockParameter>,
    pub(super) instructions: Vec<CoreInstruction>,
    pub(super) terminator: Option<CoreTerminator>,
}

pub(super) struct Builder<'a> {
    pub(super) blocks: Vec<PendingBlock>,
    pub(super) current: BlockId,
    pub(super) next_value: u32,
    pub(super) locals: BTreeMap<LocalId, ValueId>,
    pub(super) mutable_locals: BTreeMap<MutableLocalId, LocalSlotId>,
    pub(super) inputs: Vec<CoreInput>,
    pub(super) local_slots: Vec<CoreLocalSlot>,
    pub(super) closure_definitions: Vec<CoreClosureDefinition>,
    pub(super) input_ids: BTreeMap<String, InputId>,
    pub(super) metadata: BTreeMap<ValueId, CoreValueMetadata>,
    pub(super) parameter_stages: Vec<super::super::super::core::Stage>,
    pub(super) ambient: CoreValueMetadata,
    pub(super) types: TypeTableBuilder,
    pub(super) registry: &'a FunctionRegistry,
    pub(super) trusted_functions: &'a dyn Fn(&str) -> bool,
    pub(super) callable_inputs: &'a dyn Fn(&str) -> Option<CoreCallableInput>,
    pub(super) input_identity: &'a dyn Fn(&str) -> CoreInputIdentity,
    pub(super) nominal_types: &'a crate::program::TypeRegistry,
    pub(super) domain: &'a crate::program::DomainOperationRegistry,
}

impl<'a> Builder<'a> {
    pub(super) fn new(
        registry: &'a FunctionRegistry,
        trusted_functions: &'a dyn Fn(&str) -> bool,
        callable_inputs: &'a dyn Fn(&str) -> Option<CoreCallableInput>,
        input_identity: &'a dyn Fn(&str) -> CoreInputIdentity,
        nominal_types: &'a crate::program::TypeRegistry,
        domain: &'a crate::program::DomainOperationRegistry,
    ) -> Self {
        Self {
            blocks: vec![PendingBlock {
                id: BlockId::new(0),
                parameters: Vec::new(),
                instructions: Vec::new(),
                terminator: None,
            }],
            current: BlockId::new(0),
            next_value: 0,
            locals: BTreeMap::new(),
            mutable_locals: BTreeMap::new(),
            inputs: Vec::new(),
            local_slots: Vec::new(),
            closure_definitions: Vec::new(),
            input_ids: BTreeMap::new(),
            metadata: BTreeMap::new(),
            parameter_stages: Vec::new(),
            ambient: CoreValueMetadata::constant(),
            types: TypeTableBuilder::default(),
            registry,
            trusted_functions,
            callable_inputs,
            input_identity,
            nominal_types,
            domain,
        }
    }
}
