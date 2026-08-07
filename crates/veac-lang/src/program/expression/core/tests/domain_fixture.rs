use std::sync::Arc;

use super::super::metadata::EffectEvidence;
use super::super::{
    verify, BlockId, CompiledExpression, CoreBlock, CoreInstruction, CoreInstructionKind,
    CoreProgram, CoreTerminator, CoreType, CoreTypeEntry, CoreTypeId, CoreTypeTable,
    CoreValueMetadata, FunctionRegistry, Stage, ValueId, CORE_VERSION,
};
use crate::program::expression::{Value, ValueType, ValueTypeKind};
use crate::program::{
    DomainInstructionKind, DomainOperationId, DomainOperationRegistry, DomainType, OperandAxis,
};

mod transform;
pub(super) use transform::program as transform_program;

pub(super) struct Builder {
    registry: DomainOperationRegistry,
    types: Vec<CoreTypeEntry>,
    instructions: Vec<CoreInstruction>,
}

impl Builder {
    pub(super) fn new() -> Self {
        Self {
            registry: DomainOperationRegistry::standard(),
            types: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub(super) fn literal(&mut self, value: Value) -> ValueId {
        let type_id = self.intern(&value.value_type());
        self.push(
            CoreInstructionKind::Literal(value),
            type_id,
            CoreValueMetadata::constant(),
        )
    }

    pub(super) fn domain(&mut self, id: DomainOperationId, operands: Vec<ValueId>) -> ValueId {
        let contract = self.registry.lookup(id).unwrap().clone();
        let result = ValueType::domain(contract.result().domain_type().unwrap());
        let type_id = self.intern(&result);
        let mut metadata = CoreValueMetadata::constant();
        for (operand, expected) in operands.iter().zip(contract.operands()) {
            let source = &self.instructions[operand.index().unwrap()].metadata;
            match expected.axis() {
                OperandAxis::Topology => metadata.absorb_shape(source),
                OperandAxis::Leaf => metadata.absorb_leaf(source),
            }
        }
        metadata.shape_stage = metadata.shape_stage.max(Stage::Build);
        metadata.effect = metadata
            .effect
            .join(EffectEvidence::from_effect(contract.effect()));
        let kind = match contract.instruction() {
            DomainInstructionKind::DomainConstruct => CoreInstructionKind::DomainConstruct {
                opcode: id.opcode(),
                operands,
            },
            DomainInstructionKind::GraphEmit => CoreInstructionKind::GraphEmit {
                opcode: id.opcode(),
                operands,
            },
        };
        self.push(kind, type_id, metadata)
    }

    pub(super) fn finish(self, root: ValueId) -> CoreProgram {
        let result_type = self.instructions[root.index().unwrap()].type_id;
        let domain_opset = self.registry.version();
        let domain_registry_digest = self.registry.digest();
        CoreProgram {
            version: CORE_VERSION,
            domain_opset,
            domain_registry_digest,
            entry: BlockId::new(0),
            types: CoreTypeTable::new(self.types),
            nominal_definitions: Vec::new(),
            inputs: Vec::new(),
            local_slots: Vec::new(),
            closure_definitions: Vec::new(),
            blocks: vec![CoreBlock {
                id: BlockId::new(0),
                parameters: Vec::new(),
                instructions: self.instructions,
                terminator: CoreTerminator::Return {
                    value: root,
                    span: 100..101,
                },
            }],
            result_type,
        }
    }

    fn push(
        &mut self,
        kind: CoreInstructionKind,
        type_id: CoreTypeId,
        metadata: CoreValueMetadata,
    ) -> ValueId {
        let id = ValueId::new(self.instructions.len() as u32);
        self.instructions.push(CoreInstruction {
            id,
            kind,
            type_id,
            metadata,
            span: id.value() as usize..id.value() as usize + 1,
        });
        id
    }

    fn intern(&mut self, value: &ValueType) -> CoreTypeId {
        if let ValueTypeKind::List(element) = value.kind() {
            self.intern(element);
        }
        if let Some(entry) = self
            .types
            .iter()
            .find(|entry| entry.kind == CoreType::Value(value.clone()))
        {
            return entry.id;
        }
        let id = CoreTypeId::new(self.types.len() as u32);
        self.types.push(CoreTypeEntry {
            id,
            kind: CoreType::Value(value.clone()),
        });
        id
    }
}

pub(in crate::program::expression) fn compiled(program: CoreProgram) -> CompiledExpression {
    let functions = FunctionRegistry::default();
    let verified = verify(program, &functions, &[]).unwrap();
    CompiledExpression::new(verified, Arc::new(functions))
}

pub(in crate::program::expression) fn project_program() -> CoreProgram {
    let mut core = Builder::new();
    let width = core.literal(length(1080));
    let height = core.literal(length(1920));
    let canvas = core.domain(DomainOperationId::Canvas, vec![width, height]);
    let rate = core.literal(Value::Integer(30));
    let rate = core.domain(DomainOperationId::FramesPerSecond, vec![rate]);
    let project_key = core.literal(identifier("project"));
    let project = core.domain(DomainOperationId::Project, vec![project_key, canvas, rate]);
    let sequence_key = core.literal(identifier("main"));
    let sequence = core.domain(DomainOperationId::Sequence, vec![sequence_key]);
    let project = core.domain(
        DomainOperationId::ProjectWithSequence,
        vec![project, sequence],
    );
    let entry = core.literal(identifier("main"));
    let project = core.domain(DomainOperationId::ProjectEntry, vec![project, entry]);
    core.finish(project)
}

pub(super) fn resource_source_program() -> CoreProgram {
    let mut core = Builder::new();
    let key = core.literal(identifier("poster"));
    let path = core.literal(Value::Text(Arc::from("assets/poster.png")));
    let digest = core.literal(Value::Text(Arc::from(
        "0000000000000000000000000000000000000000000000000000000000000000",
    )));
    let identity = core.domain(DomainOperationId::Sha256, vec![digest]);
    let resource = core.domain(DomainOperationId::ImageResource, vec![key, path, identity]);
    let source = core.domain(DomainOperationId::Media, vec![resource]);
    core.finish(source)
}

pub(super) fn identifier(value: &str) -> Value {
    Value::Identifier(Arc::from(value))
}

fn length(value: i128) -> Value {
    Value::Length(crate::program::expression::ExactNumber::integer(value))
}

pub(super) fn result_domain(program: &CoreProgram) -> DomainType {
    program.result_type().as_domain().unwrap()
}
