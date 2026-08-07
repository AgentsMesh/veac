use super::super::super::*;
use crate::program::expression::Value;
use crate::program::VariantIndex;

pub(super) fn payload_arm(
    id: u32,
    parameter_id: u32,
    type_id: u32,
    join: u32,
    value: ValueId,
) -> CoreBlock {
    CoreBlock {
        id: BlockId::new(id),
        parameters: vec![parameter(parameter_id, type_id)],
        instructions: Vec::new(),
        terminator: CoreTerminator::Jump {
            target: BlockId::new(join),
            arguments: vec![value],
            span: 0..1,
        },
    }
}

pub(super) fn failing_arm() -> CoreBlock {
    CoreBlock {
        id: BlockId::new(2),
        parameters: vec![parameter_with_metadata(
            3,
            2,
            CoreValueMetadata::impossible(false),
        )],
        instructions: vec![
            instruction(4, CoreInstructionKind::Literal(Value::Integer(0)), 0),
            instruction(5, CoreInstructionKind::Literal(Value::Integer(1)), 0),
            instruction(6, CoreInstructionKind::Literal(Value::Integer(0)), 0),
            instruction(
                7,
                CoreInstructionKind::Range {
                    start: ValueId::new(4),
                    end: ValueId::new(5),
                    step: Some(ValueId::new(6)),
                },
                3,
            ),
        ],
        terminator: CoreTerminator::Jump {
            target: BlockId::new(4),
            arguments: vec![ValueId::new(4)],
            span: 0..1,
        },
    }
}

pub(super) fn value_arm(id: u32, value_id: u32, join: u32) -> CoreBlock {
    CoreBlock {
        id: BlockId::new(id),
        parameters: Vec::new(),
        instructions: vec![instruction(
            value_id,
            CoreInstructionKind::Literal(Value::Integer(3)),
            0,
        )],
        terminator: CoreTerminator::Jump {
            target: BlockId::new(join),
            arguments: vec![ValueId::new(value_id)],
            span: 0..1,
        },
    }
}

pub(super) fn arm(variant: u16, target: u32) -> CoreMatchArm {
    CoreMatchArm {
        variant: VariantIndex::new(variant),
        target: BlockId::new(target),
    }
}

pub(super) fn instruction(id: u32, kind: CoreInstructionKind, type_id: u32) -> CoreInstruction {
    instruction_with_metadata(id, kind, type_id, CoreValueMetadata::constant())
}

pub(super) fn instruction_with_metadata(
    id: u32,
    kind: CoreInstructionKind,
    type_id: u32,
    metadata: CoreValueMetadata,
) -> CoreInstruction {
    CoreInstruction {
        id: ValueId::new(id),
        kind,
        type_id: CoreTypeId::new(type_id),
        metadata,
        span: 0..1,
    }
}

pub(super) fn parameter(id: u32, type_id: u32) -> CoreBlockParameter {
    parameter_with_metadata(id, type_id, CoreValueMetadata::constant())
}

fn parameter_with_metadata(
    id: u32,
    type_id: u32,
    metadata: CoreValueMetadata,
) -> CoreBlockParameter {
    CoreBlockParameter {
        id: ValueId::new(id),
        type_id: CoreTypeId::new(type_id),
        metadata,
        span: 0..1,
    }
}
