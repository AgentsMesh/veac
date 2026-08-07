use std::sync::Arc;

use super::super::*;
use crate::program::expression::{Value, ValueType};
use crate::program::{StructDefinition, TypeDefinition, TypeDefinitionKind, VariantIndex};

mod builders;
mod types;
use builders::{
    arm, failing_arm, instruction, instruction_with_metadata, parameter, payload_arm, value_arm,
};
use types::{choice_definition, field, integer, integer_range, table, text};

pub(in crate::program::expression::core) fn structure() -> CoreProgram {
    let domain = crate::program::DomainOperationRegistry::standard();
    let definition = Arc::new(TypeDefinition::new(
        "types.veac",
        "Point",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![field(0, "x", integer())])),
    ));
    let nominal = ValueType::nominal(definition.type_ref().clone());
    CoreProgram {
        version: CORE_VERSION,
        domain_opset: domain.version(),
        domain_registry_digest: domain.digest(),
        entry: BlockId::new(0),
        types: table(vec![integer(), nominal]),
        nominal_definitions: vec![CoreNominalDefinition::new(definition.clone(), 0..1)],
        inputs: Vec::new(),
        local_slots: Vec::new(),
        closure_definitions: Vec::new(),
        blocks: vec![CoreBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: vec![
                instruction(0, CoreInstructionKind::Literal(Value::Integer(7)), 0),
                instruction_with_metadata(
                    1,
                    CoreInstructionKind::StructConstruct {
                        type_id: definition.type_ref().id(),
                        fields: vec![ValueId::new(0)],
                    },
                    1,
                    CoreValueMetadata::structure(&[CoreValueMetadata::constant()]),
                ),
            ],
            terminator: CoreTerminator::Return {
                value: ValueId::new(1),
                span: 0..1,
            },
        }],
        result_type: CoreTypeId::new(1),
    }
}

pub(in crate::program::expression::core) fn matching() -> CoreProgram {
    let domain = crate::program::DomainOperationRegistry::standard();
    let definition = Arc::new(choice_definition());
    let nominal = ValueType::nominal(definition.type_ref().clone());
    let blocks = vec![
        CoreBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: vec![
                instruction(0, CoreInstructionKind::Literal(Value::Integer(9)), 0),
                instruction_with_metadata(
                    1,
                    CoreInstructionKind::EnumConstruct {
                        type_id: definition.type_ref().id(),
                        variant: VariantIndex::new(0),
                        fields: vec![ValueId::new(0)],
                    },
                    1,
                    CoreValueMetadata::enumeration(
                        VariantIndex::new(0),
                        &[CoreValueMetadata::constant()],
                    ),
                ),
            ],
            terminator: CoreTerminator::Match {
                scrutinee: ValueId::new(1),
                arms: vec![arm(0, 1), arm(1, 2), arm(2, 3)],
                span: 0..1,
            },
        },
        payload_arm(1, 2, 0, 4, ValueId::new(2)),
        failing_arm(),
        value_arm(3, 8, 4),
        CoreBlock {
            id: BlockId::new(4),
            parameters: vec![parameter(9, 0)],
            instructions: Vec::new(),
            terminator: CoreTerminator::Return {
                value: ValueId::new(9),
                span: 0..1,
            },
        },
    ];
    CoreProgram {
        version: CORE_VERSION,
        domain_opset: domain.version(),
        domain_registry_digest: domain.digest(),
        entry: BlockId::new(0),
        types: table(vec![integer(), nominal, text(), integer_range()]),
        nominal_definitions: vec![CoreNominalDefinition::new(definition, 0..1)],
        inputs: Vec::new(),
        local_slots: Vec::new(),
        closure_definitions: Vec::new(),
        blocks,
        result_type: CoreTypeId::new(0),
    }
}

pub(in crate::program::expression::core) fn projection() -> CoreProgram {
    let mut program = structure();
    program.blocks[0].instructions.push(instruction(
        2,
        CoreInstructionKind::StructProject {
            structure: ValueId::new(1),
            field: crate::program::FieldIndex::new(0),
        },
        0,
    ));
    program.blocks[0].terminator = CoreTerminator::Return {
        value: ValueId::new(2),
        span: 0..1,
    };
    program.result_type = CoreTypeId::new(0);
    program
}
